//! What the Accounts pane shows: the accounts, each with its services, permissions and actions,
//! read from the module's rows alone. Nothing here knows an account or a service by name.

use super::add_account::Adding;
use super::grant::{AccountPermission, GrantFact, GrantScope, grant_path};
use super::sign_in_help::sign_in_of;
use crate::picker::{Absence, Described};
use crate::words::humanize;
use ds_settings::schema::{KeyKind, KeyPath, KeySpec};
use porter_core::SignInWay;
use porter_core::sheet::ProviderGroup;
use std::collections::BTreeSet;

/// The prefix of every row's path.
const PREFIX: &str = "accounts.";

/// Where the bring-your-own client ids sit: `accounts.clients.<issuer>`.
const CLIENTS: &str = "clients.";

/// Between an account's id and a service's kind in a row's path.
const SERVICE: &str = ".service.";

/// After an account's id, in the path of a sync switch: `accounts.<id>.sync.files`, `.sync.photos`.
const SYNC: &str = ".sync.";

/// The leaves a sync switch ends in.
const SYNC_LEAVES: [&str; 2] = ["files", "photos"];

/// Between an account's id and a grant's id in a row's path.
const GRANT: &str = ".grant.";

/// The `place` row's word for an account that is a program on this computer.
const THIS_COMPUTER: &str = "this_computer";

/// The kind of service a language model is, in a service toggle's path.
const LLM: &str = "llm";

/// The words of a service toggle.
const ON: &str = "on";
const OFF: &str = "off";

/// An account's id, as it stands in the rows' paths.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountKey(pub String);

/// How an account stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountStatus {
    /// Signed in and working.
    Ok,
    /// The sign-in has to be done again.
    NeedsReauth,
    /// An assistant that has to be signed in, inside the assistant itself.
    NeedsLogin,
    /// The service cannot be reached.
    Offline,
    /// Working, with less than was asked for.
    Limited,
    /// A state this version does not know, as the module says it.
    Other(String),
}

impl AccountStatus {
    /// The state the module's word names.
    fn of(word: &str) -> AccountStatus {
        match word {
            "ok" => AccountStatus::Ok,
            "needs_reauth" => AccountStatus::NeedsReauth,
            "needs_login" => AccountStatus::NeedsLogin,
            "offline" => AccountStatus::Offline,
            "limited" => AccountStatus::Limited,
            other => AccountStatus::Other(other.to_owned()),
        }
    }

    /// What a person reads.
    pub fn label(&self) -> String {
        match self {
            AccountStatus::Ok => "Signed in".to_owned(),
            AccountStatus::NeedsReauth | AccountStatus::NeedsLogin => "Needs sign-in".to_owned(),
            AccountStatus::Offline => "Offline".to_owned(),
            AccountStatus::Limited => "Limited".to_owned(),
            AccountStatus::Other(word) => humanize(word),
        }
    }
}

/// Where an account lives, as the module's `place` row says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Place {
    /// A program on this computer (a local runtime such as Ollama): no sign-in, found by a probe.
    ThisComputer,
    /// Anywhere else, and every account whose module says nothing of where it is.
    #[default]
    Elsewhere,
}

impl Place {
    /// The place the module's word names.
    fn of(word: &str) -> Place {
        match word {
            THIS_COMPUTER => Place::ThisComputer,
            _ => Place::Elsewhere,
        }
    }
}

/// One service of an account, a switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountService {
    /// The row, `accounts.<id>.service.<kind>`.
    pub path: KeyPath,
    /// The kind as the module names it (`mail`, `calendar`).
    pub kind: String,
    /// Whether apps may use it.
    pub on: bool,
}

impl AccountService {
    /// The words a person reads for the service.
    pub fn label(&self) -> String {
        humanize(&self.kind)
    }
}

/// One sync switch of an account: whether this computer keeps its files, or backs up its photos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSync {
    /// The row, `accounts.<id>.sync.<files|photos>`.
    pub path: KeyPath,
    /// The module's own words for it.
    pub label: String,
    /// The module's help under it ("never reads or deletes anything else"); empty when it gave
    /// none.
    pub help: String,
    /// Whether it is turned on.
    pub on: bool,
}

/// When a sign-in that expires ends, or that it did: a read-only line the module words, such as
/// "Google signs this account out on 2026-10-14".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountExpiry {
    /// The module's name for the row.
    pub label: String,
    /// What the module says about it.
    pub help: String,
    /// The module's words for the date.
    pub text: String,
}

/// The kind of account it is, as the module's `provider` row says: the provider's id and the
/// name a person reads for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountProvider {
    /// The provider's id (`fastmail`), what a mark is looked up by.
    pub id: String,
    /// The name a person reads ("Fastmail").
    pub name: String,
}

/// One account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// Its id.
    pub key: AccountKey,
    /// What the module calls it.
    pub label: String,
    /// How it stands.
    pub status: AccountStatus,
    /// Where it lives.
    pub place: Place,
    /// When its sign-in ends, for an account whose sign-in expires.
    pub expires: Option<AccountExpiry>,
    /// How it signs in again, from the module's `sign_in` row; none when the row is missing.
    pub sign_in: Option<SignInWay>,
    /// What kind of account it is, from the module's `provider` row; none when the row is missing.
    pub provider: Option<AccountProvider>,
    /// The part of the page it is listed under, from the module's `group` row; decided from
    /// the account's own rows when the row is missing.
    pub group: ProviderGroup,
    /// Its services, in the module's order.
    pub services: Vec<AccountService>,
    /// Its sync switches, in the module's order; none when syncd cannot mirror it.
    pub sync: Vec<AccountSync>,
    /// The apps' permissions for it.
    pub permissions: Vec<AccountPermission>,
    /// The row that signs in again, with its action's name.
    pub reauth: Option<(KeyPath, String)>,
    /// The row that signs an assistant's account out, with its action's name.
    pub sign_out: Option<(KeyPath, String)>,
    /// The row that removes the account, with its action's name.
    pub remove: Option<(KeyPath, String)>,
}

impl Account {
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

    /// The account as one line for a reader that is not the window: its label, the kinds of
    /// service it offers and how it stands. No id and no permission is in it.
    pub fn line(&self) -> String {
        let kinds: Vec<&str> = self.services.iter().map(|s| s.kind.as_str()).collect();
        let offers = if kinds.is_empty() {
            String::new()
        } else {
            format!(" ({})", kinds.join(", "))
        };
        format!("{}{offers}: {}", self.label, self.status.label())
    }
}

/// Whether a sign-in client field can be filled in now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientEntry {
    /// Yes.
    Open,
    /// Not yet: it is a secret, and the sign-in key it belongs to is still empty.
    WaitsForKey,
}

/// One bring-your-own sign-in key (or its secret): a text row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInClient {
    /// The row, `accounts.clients.<issuer>`.
    pub path: KeyPath,
    /// What the module calls it.
    pub label: String,
    /// What the module says about it.
    pub help: String,
    /// The text held now; empty when the build's own is used.
    pub value: String,
    /// Whether it can be filled in now.
    pub entry: ClientEntry,
}

/// What a secret row's path ends in, after the path of the key it belongs to.
const SECRET_LEAF: &str = ".secret";

impl SignInClient {
    /// The line the person reads when they try to fill in a secret before its key.
    pub const KEY_FIRST: &'static str = "Set the sign-in key first, then its secret.";
}

/// Marks each secret whose key is still empty as not yet fillable.
fn wait_for_keys(clients: &mut [SignInClient]) {
    let empty_keys: Vec<String> = clients
        .iter()
        .filter(|c| c.value.is_empty() && !c.path.0.ends_with(SECRET_LEAF))
        .map(|c| c.path.0.clone())
        .collect();
    for client in clients.iter_mut() {
        let waits = client
            .path
            .0
            .strip_suffix(SECRET_LEAF)
            .is_some_and(|key| empty_keys.iter().any(|empty| empty == key));
        if waits {
            client.entry = ClientEntry::WaitsForKey;
        }
    }
}

/// Everything the module described, read as accounts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountsPane {
    /// The accounts, in the order their rows first appear.
    pub accounts: Vec<Account>,
    /// The sign-in client ids.
    pub clients: Vec<SignInClient>,
}

/// The group a `group` row's word names; none for a word this build does not know.
fn group_of(word: &str) -> Option<ProviderGroup> {
    ProviderGroup::ALL
        .into_iter()
        .find(|group| group.slug() == word)
}

/// What a row of an account is, by its kind and where its path puts it.
pub(super) enum Row<'a> {
    Label(&'a str),
    State(&'a str),
    Place(&'a str),
    Expires(&'a str),
    SignIn(&'a str),
    Provider(&'a str),
    Group(&'a str),
    GrantSince(&'a str, &'a str),
    GrantScope(&'a str, &'a str),
    AccountService(&'a str, bool),
    Sync(bool),
    Grant,
    Reauth,
    SignOut,
    Remove,
}

/// The account a row belongs to and what the row is. An id may hold dots, so the row's kind
/// and the words between the id and the rest decide, not a count of segments.
pub(super) fn classify<'a>(
    spec: &'a KeySpec,
    described: &Described,
) -> Option<(AccountKey, Row<'a>)> {
    let rest = spec.path.0.strip_prefix(PREFIX)?;
    let held = |path: &KeyPath| described.values.get(path).is_some_and(|word| word == ON);
    match &spec.kind {
        KeyKind::Toggle { .. } => {
            if let Some((id, kind)) = rest.rsplit_once(SERVICE) {
                return Some((
                    AccountKey(id.to_owned()),
                    Row::AccountService(kind, held(&spec.path)),
                ));
            }
            let (id, leaf) = rest.rsplit_once(SYNC)?;
            SYNC_LEAVES
                .contains(&leaf)
                .then(|| (AccountKey(id.to_owned()), Row::Sync(held(&spec.path))))
        }
        KeyKind::Fixed { variant } => {
            if let Some((id, tail)) = rest.split_once(GRANT) {
                let row = match tail.rsplit_once('.')? {
                    (grant, "since") => Row::GrantSince(grant, variant),
                    (grant, "scope") => Row::GrantScope(grant, variant),
                    _ => return None,
                };
                return Some((AccountKey(id.to_owned()), row));
            }
            let (id, leaf) = rest.rsplit_once('.')?;
            let row = match leaf {
                "group" => Row::Group(variant),
                "label" => Row::Label(variant),
                "state" => Row::State(variant),
                "place" => Row::Place(variant),
                "expires" => Row::Expires(variant),
                "sign_in" => Row::SignIn(variant),
                "provider" => Row::Provider(variant),
                _ => return None,
            };
            Some((AccountKey(id.to_owned()), row))
        }
        KeyKind::Live { .. } => {
            if let Some((id, _)) = rest.split_once(GRANT) {
                return Some((AccountKey(id.to_owned()), Row::Grant));
            }
            let (id, leaf) = rest.rsplit_once('.')?;
            let row = match leaf {
                "reauth" => Row::Reauth,
                "sign_out" => Row::SignOut,
                "remove" => Row::Remove,
                _ => return None,
            };
            Some((AccountKey(id.to_owned()), row))
        }
        _ => None,
    }
}

fn account_of<'a>(accounts: &'a mut Vec<Account>, key: &AccountKey) -> &'a mut Account {
    let at = accounts
        .iter()
        .position(|account| &account.key == key)
        .unwrap_or_else(|| {
            accounts.push(Account {
                key: key.clone(),
                label: key.0.clone(),
                status: AccountStatus::Ok,
                place: Place::default(),
                expires: None,
                sign_in: None,
                provider: None,
                group: ProviderGroup::Internet,
                services: Vec::new(),
                sync: Vec::new(),
                permissions: Vec::new(),
                reauth: None,
                sign_out: None,
                remove: None,
            });
            accounts.len() - 1
        });
    &mut accounts[at]
}

pub(super) fn action_name(spec: &KeySpec) -> String {
    match &spec.kind {
        KeyKind::Live { action } => action.label.0.clone(),
        _ => spec.label.0.clone(),
    }
}

impl AccountsPane {
    /// The pane `described` makes.
    pub fn of(described: &Described) -> AccountsPane {
        let mut pane = AccountsPane::default();
        let mut told = BTreeSet::new();
        let mut facts = Vec::new();
        for spec in &described.keys {
            if let Some(id) = spec.path.0.strip_prefix(PREFIX)
                && id.starts_with(CLIENTS)
                && matches!(spec.kind, KeyKind::Text)
            {
                pane.clients.push(SignInClient {
                    path: spec.path.clone(),
                    label: spec.label.0.clone(),
                    help: spec.help.0.clone(),
                    value: described
                        .values
                        .get(&spec.path)
                        .cloned()
                        .unwrap_or_default(),
                    entry: ClientEntry::Open,
                });
                continue;
            }
            let Some((key, row)) = classify(spec, described) else {
                continue;
            };
            let account = account_of(&mut pane.accounts, &key);
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
                    facts.push((grant_path(&key, grant), GrantFact::Since(day.to_owned())));
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
                Row::Grant => account.permissions.push(AccountPermission {
                    path: spec.path.clone(),
                    label: spec.label.0.clone(),
                    action: action_name(spec),
                    since: None,
                    scope: None,
                }),
                Row::Reauth => account.reauth = Some((spec.path.clone(), action_name(spec))),
                Row::SignOut => account.sign_out = Some((spec.path.clone(), action_name(spec))),
                Row::Remove => account.remove = Some((spec.path.clone(), action_name(spec))),
            }
        }
        for account in &mut pane.accounts {
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
        wait_for_keys(&mut pane.clients);
        pane
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

    /// The row at `path`: the account that holds it, for the pane to say what it is changing.
    pub fn owner(&self, path: &KeyPath) -> Option<&Account> {
        self.accounts.iter().find(|account| {
            account.services.iter().any(|s| &s.path == path)
                || account.sync.iter().any(|s| &s.path == path)
                || account.permissions.iter().any(|p| &p.path == path)
                || account.reauth.as_ref().is_some_and(|(p, _)| p == path)
                || account.sign_out.as_ref().is_some_and(|(p, _)| p == path)
                || account.remove.as_ref().is_some_and(|(p, _)| p == path)
        })
    }
}

/// Which of the pane's two screens is up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountsScreen {
    /// The list of accounts.
    List,
    /// One account.
    Detail(AccountKey),
}

/// What an action waiting for the person's answer does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// Remove the account.
    Remove,
    /// Sign an assistant's account out.
    SignOut,
}

/// Whether this computer keeps files for the account that a removal would delete too: a sync
/// switch of the account is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalFiles {
    /// A sync switch is on: files kept for the account go with it.
    Kept,
    /// None are kept.
    None,
}

/// A removal or sign-out waiting for the person's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovalQuestion {
    /// What is asked.
    pub ask: Ask,
    /// The row that runs the action.
    pub path: KeyPath,
    /// The account's label, for the question.
    pub account: String,
    /// Whether files kept on this computer for the account are deleted with it.
    pub files: LocalFiles,
}

/// A pane that is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountsReady {
    /// What the module described.
    pub pane: AccountsPane,
    /// The screen up.
    pub screen: AccountsScreen,
    /// The account the list rests on: the one its foot's minus acts on.
    pub selected: Option<AccountKey>,
    /// The row being set or run: one at a time.
    pub pending: Option<KeyPath>,
    /// The removal asked about, if any.
    pub confirming: Option<RemovalQuestion>,
    /// Whether the add sheet is open.
    pub adding: Adding,
}

/// Where the pane stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AccountsState {
    /// Looking for the module.
    #[default]
    Probing,
    /// The module is not there, and why.
    Unavailable(Absence),
    /// The module answered.
    Ready(AccountsReady),
}

impl AccountsReady {
    /// A pane showing the list.
    pub(crate) fn new(pane: AccountsPane) -> AccountsReady {
        AccountsReady {
            pane,
            screen: AccountsScreen::List,
            selected: None,
            pending: None,
            confirming: None,
            adding: Adding::Idle,
        }
    }

    /// The account the detail screen shows, when it is up and the account still exists.
    pub fn open(&self) -> Option<&Account> {
        match &self.screen {
            AccountsScreen::Detail(key) => self.pane.account(key),
            AccountsScreen::List => None,
        }
    }
}

/// The word a service toggle stores for `on`.
pub(crate) fn toggle_word(on: bool) -> &'static str {
    if on { ON } else { OFF }
}
