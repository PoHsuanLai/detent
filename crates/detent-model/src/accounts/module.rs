//! The three modules the Accounts pane reads: which one a row came from, and what each has said.

use super::sign_in_client::standing_only;
use crate::Absence;
use crate::Live;
use crate::picker::Described;
use ds_settings::schema::KeyPath;

/// A live settings module the pane reads. Each holds its own accounts, and a row's action goes
/// back to the module the row came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleId {
    /// Identities, AI keys and assistant accounts (with the permits apps hold for them), and
    /// Tailscale.
    Accountd,
    /// The month's spend and cap of the AI accounts accountd lists; it lists no account itself.
    Inferd,
    /// The accounts sync signed in for itself, with their sync switches.
    Syncd,
}

impl ModuleId {
    /// Every module, in the order the pane lists their accounts.
    pub const ALL: [ModuleId; 3] = [ModuleId::Accountd, ModuleId::Inferd, ModuleId::Syncd];
}

/// A row of one module: the same path may be in two modules, so a row is named with its module.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleRow {
    /// The module the row is in.
    pub module: ModuleId,
    /// The row's path.
    pub path: KeyPath,
}

impl ModuleRow {
    /// The row at `path` of `module`.
    pub fn new(module: ModuleId, path: KeyPath) -> ModuleRow {
        ModuleRow { module, path }
    }
}

/// What one module has told the pane: nothing yet, why it is not there, or its rows as last read.
pub type Source = Live<Described>;

/// What the three modules have told the pane.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sources {
    accountd: Source,
    inferd: Source,
    syncd: Source,
}

impl Sources {
    fn slot(&mut self, module: ModuleId) -> &mut Source {
        match module {
            ModuleId::Accountd => &mut self.accountd,
            ModuleId::Inferd => &mut self.inferd,
            ModuleId::Syncd => &mut self.syncd,
        }
    }

    /// What `module` last said.
    pub fn of(&self, module: ModuleId) -> &Source {
        match module {
            ModuleId::Accountd => &self.accountd,
            ModuleId::Inferd => &self.inferd,
            ModuleId::Syncd => &self.syncd,
        }
    }

    /// Takes in what `module` found: its rows, or that it is not there.
    pub fn hear(&mut self, module: ModuleId, found: Result<Described, Absence>) {
        *self.slot(module) = match found {
            Ok(described) => Source::Ready(standing_only(described)),
            Err(absence) => Source::Unavailable(absence),
        };
    }

    /// Whether `module`'s rows are held.
    pub fn has_read(&self, module: ModuleId) -> bool {
        matches!(self.of(module), Source::Ready(_))
    }

    /// The pane the modules that answered make; none when none did.
    pub fn pane(&self) -> Option<crate::accounts::AccountsPane> {
        let read: Vec<(ModuleId, &Described)> = ModuleId::ALL
            .into_iter()
            .filter_map(|module| match self.of(module) {
                Source::Ready(described) => Some((module, described)),
                _ => None,
            })
            .collect();
        (!read.is_empty()).then(|| crate::accounts::AccountsPane::of_modules(read))
    }

    /// Why nothing is shown, once every module has answered and none is there: accountd's
    /// reason first. `None` while one is still being looked for, or when one is there.
    pub fn absence(&self) -> Option<Absence> {
        let mut first = None;
        for module in ModuleId::ALL {
            match self.of(module) {
                Source::Looking | Source::Ready(_) => return None,
                Source::Unavailable(absence) => {
                    first.get_or_insert(*absence);
                }
            }
        }
        first
    }
}
