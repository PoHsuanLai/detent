//! What the Accounts pane shows, as data: the accounts, each with its services, permissions and
//! actions, and where the pane stands. Nothing here knows an account or a service by name; the
//! rows are read into these in `pane.rs`.

use super::grant::AccountPermission;
use super::module::{ModuleId, ModuleRow, Sources};
use super::sign_in_client::SignInClient;
use super::spend::AccountSpend;
use crate::Absence;
use ds_settings::schema::KeyPath;
use porter_core::SignInWay;
use porter_sheet::ProviderGroup;

/// Before an id in the key of an account that is syncd's: an id never holds a `/`, so the same
/// id in two modules is two accounts.
const SYNC_KEY: &str = "sync/";

/// The `place` row's word for an account that is a program on this computer.
const THIS_COMPUTER: &str = "this_computer";

/// The words a switch is stored as.
const ON: &str = "on";
const OFF: &str = "off";

/// An account's id, as it stands in the rows' paths.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountKey(pub String);

impl AccountKey {
    /// The key of the account `id` of `module`.
    pub fn of(module: ModuleId, id: &str) -> AccountKey {
        match module {
            ModuleId::Syncd => AccountKey(format!("{SYNC_KEY}{id}")),
            ModuleId::Accountd | ModuleId::Inferd => AccountKey(id.to_owned()),
        }
    }

    /// The id as the module writes it in its rows' paths.
    pub fn id(&self) -> &str {
        self.0.strip_prefix(SYNC_KEY).unwrap_or(&self.0)
    }
}

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
    pub(super) fn of(word: &str) -> AccountStatus {
        match word {
            "ok" => AccountStatus::Ok,
            "needs_reauth" => AccountStatus::NeedsReauth,
            "needs_login" => AccountStatus::NeedsLogin,
            "offline" => AccountStatus::Offline,
            "limited" => AccountStatus::Limited,
            other => AccountStatus::Other(other.to_owned()),
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
    pub(super) fn of(word: &str) -> Place {
        match word {
            THIS_COMPUTER => Place::ThisComputer,
            _ => Place::Elsewhere,
        }
    }
}

/// Whether a switch is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchState {
    /// Apps may use it, or it is kept.
    On,
    /// They may not, or it is not.
    Off,
}

impl SwitchState {
    /// The state a row's stored word names; anything but `on` is off.
    pub(super) fn of_word(word: &str) -> SwitchState {
        if word == ON {
            SwitchState::On
        } else {
            SwitchState::Off
        }
    }

    /// The word the module stores for this state.
    pub(crate) fn word(self) -> &'static str {
        match self {
            SwitchState::On => ON,
            SwitchState::Off => OFF,
        }
    }

    /// Whether it is on.
    pub fn is_on(self) -> bool {
        self == SwitchState::On
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
    pub on: SwitchState,
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
    pub on: SwitchState,
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
    /// The module its rows are in: an action on one goes back to it.
    pub module: ModuleId,
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
    /// Its "Show in Files" switch: the whole drive in Files, downloaded as it is opened. Only for
    /// an account whose drive the sync service can show; its help carries the service's line when
    /// turning it on did not end on.
    pub show_in_files: Option<AccountSync>,
    /// The apps' permissions for it.
    pub permissions: Vec<AccountPermission>,
    /// The row that signs in again, with its action's name.
    pub reauth: Option<(KeyPath, String)>,
    /// The row that signs an assistant's account out, with its action's name.
    pub sign_out: Option<(KeyPath, String)>,
    /// The row that removes the account, with its action's name.
    pub remove: Option<(KeyPath, String)>,
    /// The apps using the identity, by the names a person knows them by; read-only.
    pub used_by: Vec<String>,
    /// This month's cost and the cap, for an account that costs money; empty when the module
    /// publishes neither.
    pub spend: AccountSpend,
}

impl Account {
    /// The row at `path` of this account, named with its module.
    pub fn row(&self, path: &KeyPath) -> ModuleRow {
        ModuleRow::new(self.module, path.clone())
    }
}

/// The row of syncd's module that starts "Continue as" for a new account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncAdd {
    /// The row.
    pub path: KeyPath,
    /// The action's name.
    pub name: String,
}

impl SyncAdd {
    /// The row, named with its module.
    pub fn row(&self) -> ModuleRow {
        ModuleRow::new(ModuleId::Syncd, self.path.clone())
    }
}

/// Everything the modules described, read as accounts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccountsPane {
    /// The accounts, module by module, each in the order its rows first appear.
    pub accounts: Vec<Account>,
    /// The sign-in client ids.
    pub clients: Vec<SignInClient>,
    /// The modules that answered: a part of the pane is left out for one that did not.
    pub modules: Vec<ModuleId>,
    /// Syncd's row for a new account, when it has one.
    pub sync_add: Option<SyncAdd>,
}

/// What an action waiting for the person's answer does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ask {
    /// Remove the account.
    Remove,
    /// Sign an assistant's account out.
    SignOut,
}

/// A removal or sign-out waiting for the person's answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovalQuestion {
    /// What is asked.
    pub ask: Ask,
    /// The row that runs the action.
    pub row: ModuleRow,
    /// The account's label, for the question.
    pub account: String,
    /// Whether files kept on this computer for the account are deleted with it.
    pub files_kept: bool,
}

/// A pane that is there.
#[derive(Debug, Clone, PartialEq)]
pub struct AccountsReady {
    /// What the module described.
    pub pane: AccountsPane,
    /// The account whose detail screen is up; none while the list is.
    pub detail: Option<AccountKey>,
    /// The account the list rests on: the one its foot's minus acts on.
    pub selected: Option<AccountKey>,
    /// The row being set or run: one at a time.
    pub pending: Option<ModuleRow>,
    /// The removal asked about, if any.
    pub confirming: Option<RemovalQuestion>,
    /// Whether the add sheet is open.
    pub sheet_open: bool,
    /// What the modules told.
    pub sources: Sources,
}

/// Where the pane stands.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountsState {
    /// Looking for the modules.
    Probing(Sources),
    /// Every module is absent: why, by the first.
    Unavailable(Absence, Sources),
    /// At least one module answered.
    Ready(Box<AccountsReady>),
}

impl Default for AccountsState {
    fn default() -> Self {
        AccountsState::Probing(Sources::default())
    }
}

impl AccountsReady {
    /// A pane showing the list.
    pub(crate) fn new(pane: AccountsPane, sources: Sources) -> AccountsReady {
        AccountsReady {
            pane,
            detail: None,
            selected: None,
            pending: None,
            confirming: None,
            sheet_open: false,
            sources,
        }
    }

    /// The account the detail screen shows, when it is up and the account still exists.
    pub fn open(&self) -> Option<&Account> {
        self.detail.as_ref().and_then(|key| self.pane.account(key))
    }
}
