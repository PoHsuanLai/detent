//! Which daemon a live module belongs to.

use porter_dbus::{ACCOUNTS_BUS, ACCOUNTS_SETTINGS_PATH, INFERENCE_BUS, INFERENCE_SETTINGS_PATH};

/// A daemon that serves a live settings module: its bus name and the module's object path.
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
}
