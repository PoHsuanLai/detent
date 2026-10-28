//! Reading the modules' rows into the pane: each row is told to its account, and the pane's
//! questions (which accounts, which one owns a row) are answered from what was read.

use super::day::CalendarDay;
use super::grant::{AccountPermission, GrantFact, GrantScope, grant_path};
use super::model::{
    Account, AccountExpiry, AccountKey, AccountProvider, AccountService, AccountStatus,
    AccountSync, AccountsPane, Place, SyncAdd,
};
use super::module::{ModuleId, ModuleRow};
use super::rows::{CLIENTS, LLM, PREFIX, Row, SYNC_ADD, action_name, classify, sign_in_of};
use super::sign_in_client::{SignInClient, wait_for_keys};
use super::spend::{AccountSpend, Cents};
use crate::picker::Described;
use ds_settings::schema::{KeyKind, KeyPath};
use porter_core::SignInWay;
use porter_sheet::ProviderGroup;
use std::collections::BTreeSet;

impl Account {
    /// Whether `path` is a row of this account that can be set or run.
    fn owns(&self, path: &KeyPath) -> bool {
        self.services.iter().any(|s| &s.path == path)
            || self.sync.iter().any(|s| &s.path == path)
            || self.show_in_files.as_ref().is_some_and(|s| &s.path == path)
            || self.permissions.iter().any(|p| &p.path == path)
            || self.reauth.as_ref().is_some_and(|(p, _)| p == path)
            || self.sign_out.as_ref().is_some_and(|(p, _)| p == path)
            || self.remove.as_ref().is_some_and(|(p, _)| p == path)
    }

    /// Whether this is a language-model runtime on this computer.
    pub fn is_local_runtime(&self) -> bool {
        self.place == Place::ThisComputer && self.services.iter().any(|s| s.kind == LLM)
    }

    /// Drops the rows the account's state makes no sense for, whatever the module sent: a
    /// program on this computer has nothing to sign in to, and an assistant that is not signed in
    /// has nothing to sign out of.
    fn keep_offered_rows(&mut self) {
        if self.place == Place::ThisComputer {
            self.reauth = None;
        }
        if self.status == AccountStatus::NeedsLogin {
            self.sign_out = None;
        }
    }

    /// The group the account's own rows put it in, when the module sent no `group` row: the
    /// assistants that sign themselves in, then what runs on this computer, then the rest.
    fn fallback_group(&self) -> ProviderGroup {
        if self.sign_in == Some(SignInWay::Agent) {
            ProviderGroup::Agent
        } else if self.place == Place::ThisComputer {
            ProviderGroup::Intelligence
        } else {
            ProviderGroup::Internet
        }
    }
}

/// The group a `group` row's word names; none for a word this build does not know.
fn group_of(word: &str) -> Option<ProviderGroup> {
    porter_ids::word::from_word(word)
}

fn account_of<'a>(
    accounts: &'a mut Vec<Account>,
    module: ModuleId,
    key: &AccountKey,
) -> &'a mut Account {
    let at = accounts
        .iter()
        .position(|account| account.module == module && &account.key == key)
        .unwrap_or_else(|| {
            accounts.push(Account {
                key: key.clone(),
                module,
                label: key.id().to_owned(),
                status: AccountStatus::Ok,
                place: Place::default(),
                expires: None,
                sign_in: None,
                provider: None,
                group: ProviderGroup::Internet,
                services: Vec::new(),
                sync: Vec::new(),
                show_in_files: None,
                permissions: Vec::new(),
                reauth: None,
                sign_out: None,
                remove: None,
                used_by: Vec::new(),
                spend: AccountSpend::default(),
            });
            accounts.len() - 1
        });
    &mut accounts[at]
}

impl AccountsPane {
    /// The pane the rows of one module make.
    pub fn of(module: ModuleId, described: &Described) -> AccountsPane {
        AccountsPane::of_modules([(module, described)])
    }

    /// The pane the modules' rows make together: each module's accounts in its own order,
    /// accountd's first.
    pub fn of_modules<'a>(
        modules: impl IntoIterator<Item = (ModuleId, &'a Described)>,
    ) -> AccountsPane {
        let mut pane = AccountsPane::default();
        for (module, described) in modules {
            pane.absorb(module, described);
        }
        wait_for_keys(&mut pane.clients);
        pane
    }

    fn absorb(&mut self, module: ModuleId, described: &Described) {
        let pane = self;
        pane.modules.push(module);
        let mut told = BTreeSet::new();
        let mut facts = Vec::new();
        for spec in &described.keys {
            if let Some(id) = spec.path.0.strip_prefix(PREFIX)
                && id.starts_with(CLIENTS)
                && matches!(spec.kind, KeyKind::Text)
            {
                pane.clients.push(SignInClient::read(
                    module,
                    spec,
                    described.values.get(&spec.path).map_or("", String::as_str),
                ));
                continue;
            }
            if module == ModuleId::Syncd
                && spec.path.0 == SYNC_ADD
                && matches!(spec.kind, KeyKind::Live { .. })
            {
                pane.sync_add = Some(SyncAdd {
                    path: spec.path.clone(),
                    name: action_name(spec),
                });
                continue;
            }
            let Some((key, row)) = classify(module, spec, described) else {
                continue;
            };
            let account = account_of(&mut pane.accounts, module, &key);
            match row {
                Row::Label(label) => account.label = label.to_owned(),
                Row::State(word) => account.status = AccountStatus::of(word),
                Row::Place(word) => account.place = Place::of(word),
                Row::SignIn(word) => account.sign_in = sign_in_of(word),
                Row::Provider(id) => {
                    account.provider = Some(AccountProvider {
                        id: id.to_owned(),
                        name: spec.labels.of(id).unwrap_or(id).to_owned(),
                    });
                }
                Row::Group(word) => {
                    if let Some(group) = group_of(word) {
                        account.group = group;
                        told.insert(key.clone());
                    }
                }
                Row::GrantSince(grant, day) => {
                    facts.extend(
                        CalendarDay::parse(day)
                            .map(|day| (grant_path(&key, grant), GrantFact::Since(day))),
                    );
                }
                Row::GrantScope(grant, word) => {
                    facts.extend(
                        GrantScope::of(word)
                            .map(|scope| (grant_path(&key, grant), GrantFact::Scope(scope))),
                    );
                }
                Row::Expires(text) => {
                    account.expires = Some(AccountExpiry {
                        label: spec.label.0.clone(),
                        help: spec.help.0.clone(),
                        text: text.to_owned(),
                    });
                }
                // An account with several models sends one service row per model claim: the
                // same path again is the same switch.
                Row::AccountService(..) if account.services.iter().any(|s| s.path == spec.path) => {
                }
                Row::AccountService(kind, on) => account.services.push(AccountService {
                    path: spec.path.clone(),
                    kind: kind.to_owned(),
                    on,
                }),
                Row::Sync(on) => account.sync.push(AccountSync {
                    path: spec.path.clone(),
                    label: spec.label.0.clone(),
                    help: spec.help.0.clone(),
                    on,
                }),
                Row::ShowInFiles(on) => {
                    account.show_in_files = Some(AccountSync {
                        path: spec.path.clone(),
                        label: spec.label.0.clone(),
                        help: spec.help.0.clone(),
                        on,
                    });
                }
                Row::Grant => account.permissions.push(AccountPermission {
                    path: spec.path.clone(),
                    label: spec.label.0.clone(),
                    action: action_name(spec),
                    since: None,
                    scope: None,
                }),
                Row::UsedBy(app) => account.used_by.push(app.to_owned()),
                Row::SpendMonth(word) => account.spend.spent = Cents::parse(word),
                Row::SpendCap(word) => account.spend.cap = Cents::parse(word),
                Row::Reauth => account.reauth = Some((spec.path.clone(), action_name(spec))),
                Row::SignOut => account.sign_out = Some((spec.path.clone(), action_name(spec))),
                Row::Remove => account.remove = Some((spec.path.clone(), action_name(spec))),
            }
        }
        for account in pane.accounts.iter_mut().filter(|a| a.module == module) {
            if !told.contains(&account.key) {
                account.group = account.fallback_group();
            }
            account.keep_offered_rows();
            for permission in &mut account.permissions {
                let path = permission.path.clone();
                for (_, fact) in facts.iter().filter(|(row, _)| *row == path) {
                    permission.learn(fact);
                }
            }
        }
    }

    /// The language-model runtimes on this computer, in the module's order: running or not.
    pub fn local_runtimes(&self) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(|a| a.is_local_runtime())
    }

    /// The accounts of `group`, in the module's order.
    pub fn in_group(&self, group: ProviderGroup) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(move |a| a.group == group)
    }

    /// The account with `key`.
    pub fn account(&self, key: &AccountKey) -> Option<&Account> {
        self.accounts.iter().find(|account| &account.key == key)
    }

    /// The row `row`: the account that holds it, for the pane to say what it is changing.
    pub fn owner(&self, row: &ModuleRow) -> Option<&Account> {
        self.accounts
            .iter()
            .find(|account| account.module == row.module && account.owns(&row.path))
    }

    /// Whether `module` answered, so its part of the pane is there.
    pub fn has(&self, module: ModuleId) -> bool {
        self.modules.contains(&module)
    }
}
