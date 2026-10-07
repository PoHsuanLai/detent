//! What the Accounts pane shows: the accounts, each with its services, permissions and actions,
//! read from the module's rows alone. Nothing here knows an account or a service by name.

use crate::picker::{Absence, Described};
use crate::words::humanize;
use ds_settings::schema::{KeyKind, KeyPath, KeySpec};

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
    /// A coding agent that has to be signed in, inside the agent itself.
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
    /// Whether it is turned on.
    pub on: bool,
}

/// One permission an app holds for the account: a sentence and the action that takes it back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountPermission {
    /// The row, `accounts.<id>.grant.<grant>`.
    pub path: KeyPath,
    /// What the app may or may not do.
    pub label: String,
    /// The action's name (`Revoke`).
    pub action: String,
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
    /// Its services, in the module's order.
    pub services: Vec<AccountService>,
    /// Its sync switches, in the module's order; none when syncd cannot mirror it.
    pub sync: Vec<AccountSync>,
    /// The apps' permissions for it.
    pub permissions: Vec<AccountPermission>,
    /// The row that signs in again, with its action's name.
    pub reauth: Option<(KeyPath, String)>,
    /// The row that signs a coding agent's account out, with its action's name.
    pub sign_out: Option<(KeyPath, String)>,
    /// The row that removes the account, with its action's name.
    pub remove: Option<(KeyPath, String)>,
}

impl Account {
    /// Whether this is a language-model runtime on this computer.
    pub fn is_local_runtime(&self) -> bool {
        self.place == Place::ThisComputer && self.services.iter().any(|s| s.kind == LLM)
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

/// One bring-your-own sign-in client id: a text row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInClient {
    /// The row, `accounts.clients.<issuer>`.
    pub path: KeyPath,
    /// What the module calls it.
    pub label: String,
    /// What the module says about it.
    pub help: String,
    /// The id held now; empty when the build's own is used.
    pub value: String,
}

/// Everything the module described, read as accounts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountsPane {
    /// The accounts, in the order their rows first appear.
    pub accounts: Vec<Account>,
    /// The sign-in client ids.
    pub clients: Vec<SignInClient>,
}

/// What a row of an account is, by its kind and where its path puts it.
pub(super) enum Row<'a> {
    Label(&'a str),
    State(&'a str),
    Place(&'a str),
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
            let (id, leaf) = rest.rsplit_once('.')?;
            let row = match leaf {
                "label" => Row::Label(variant),
                "state" => Row::State(variant),
                "place" => Row::Place(variant),
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

fn action_name(spec: &KeySpec) -> String {
    match &spec.kind {
        KeyKind::Live { action } => action.label.0.clone(),
        _ => spec.label.0.clone(),
    }
}

impl AccountsPane {
    /// The pane `described` makes.
    pub fn of(described: &Described) -> AccountsPane {
        let mut pane = AccountsPane::default();
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
                    on,
                }),
                Row::Grant => account.permissions.push(AccountPermission {
                    path: spec.path.clone(),
                    label: spec.label.0.clone(),
                    action: action_name(spec),
                }),
                Row::Reauth => account.reauth = Some((spec.path.clone(), action_name(spec))),
                Row::SignOut => account.sign_out = Some((spec.path.clone(), action_name(spec))),
                Row::Remove => account.remove = Some((spec.path.clone(), action_name(spec))),
            }
        }
        pane
    }

    /// The language-model runtimes on this computer, in the module's order: running or not.
    pub fn local_runtimes(&self) -> impl Iterator<Item = &Account> {
        self.accounts.iter().filter(|a| a.is_local_runtime())
    }

    /// The account with `key`.
    pub fn account(&self, key: &AccountKey) -> Option<&Account> {
        self.accounts.iter().find(|account| &account.key == key)
    }

    /// The row at `path`: the account that holds it, for the pane to say what it is changing.
    pub(crate) fn owner(&self, path: &KeyPath) -> Option<&Account> {
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
    /// Sign a coding agent's account out.
    SignOut,
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
}

/// A pane that is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountsReady {
    /// What the module described.
    pub pane: AccountsPane,
    /// The screen up.
    pub screen: AccountsScreen,
    /// The row being set or run: one at a time.
    pub pending: Option<KeyPath>,
    /// The removal asked about, if any.
    pub confirming: Option<RemovalQuestion>,
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
            pending: None,
            confirming: None,
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
