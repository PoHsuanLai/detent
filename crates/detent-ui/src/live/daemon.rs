//! Which daemon a live module belongs to.

use detent_model::ModuleId;
use porter_dbus::names::{
    ACCOUNTS_BUS, ACCOUNTS_SETTINGS_PATH, INFERENCE_BUS, INFERENCE_SETTINGS_PATH, SYNC_BUS,
    SYNC_PATH,
};

/// The object path of syncd's settings module.
const SYNC_SETTINGS_PATH: &str = "/org/quire/Sync1/settings";

/// A daemon on the bus: its name and its object path (for a live settings module, the module's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Daemon {
    /// The bus name the daemon owns.
    pub name: &'static str,
    /// The module's object path.
    pub path: String,
}

impl Daemon {
    /// inferd, whose module holds the model picker's rows.
    pub fn inferd() -> Daemon {
        Daemon {
            name: INFERENCE_BUS,
            path: INFERENCE_SETTINGS_PATH.to_owned(),
        }
    }

    /// accountd, whose module holds the accounts and their permissions.
    pub fn accountd() -> Daemon {
        Daemon {
            name: ACCOUNTS_BUS,
            path: ACCOUNTS_SETTINGS_PATH.to_owned(),
        }
    }

    /// The settings module of `module`: the three the Accounts pane reads.
    pub fn of(module: ModuleId) -> Daemon {
        match module {
            ModuleId::Accountd => Daemon::accountd(),
            ModuleId::Inferd => Daemon::inferd(),
            ModuleId::Syncd => Daemon::syncd_settings(),
        }
    }

    /// syncd's settings module: its accounts and their sync switches. The module is at the
    /// path below the one syncd tells deletions on.
    pub fn syncd_settings() -> Daemon {
        Daemon {
            name: SYNC_BUS,
            path: SYNC_SETTINGS_PATH.to_owned(),
        }
    }

    /// syncd, which holds a deletion for the person's word and tells it (`org.quire.Sync1`).
    pub fn syncd() -> Daemon {
        Daemon {
            name: SYNC_BUS,
            path: SYNC_PATH.to_owned(),
        }
    }
}
