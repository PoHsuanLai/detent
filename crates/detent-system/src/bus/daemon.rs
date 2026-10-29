//! Which daemon a live module belongs to.

use detent_callers::COMPOSITOR_BUS;
use detent_model::ModuleId;
use porter_dbus::sync::{SYNC_BUS, SYNC_PATH};
use porter_wire::names::{ACCOUNTS_BUS, ACCOUNTS_SETTINGS_PATH, INFERENCE_BUS};
use steward_dbus::inference::INFERENCE_SETTINGS_PATH;

/// The object path of syncd's settings module.
const SYNC_SETTINGS_PATH: &str = "/org/quire/Sync1/settings";

/// The object path of the compositor's Privacy module.
const PRIVACY_PATH: &str = "/org/quire/Compositor1/privacy";

/// A daemon on the bus: its name and its object path (for a live settings module, the module's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Daemon {
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

    /// The compositor, whose module lists which program may use which power.
    pub fn compositor() -> Daemon {
        Daemon {
            name: COMPOSITOR_BUS,
            path: PRIVACY_PATH.to_owned(),
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

#[cfg(test)]
mod tests {
    use super::Daemon;
    use detent_callers::{ACCOUNTS_BUS, COMPOSITOR_BUS, INFERENCE_BUS, SYNC_BUS, required_role};

    /// The names the panes call and the names the owner check knows the roles of are one list:
    /// a service whose name drifted would be refused as a stranger, and the page would say so.
    #[test]
    fn every_daemon_a_pane_reads_is_a_service_the_owner_check_knows() {
        assert_eq!(Daemon::accountd().name, ACCOUNTS_BUS);
        assert_eq!(Daemon::inferd().name, INFERENCE_BUS);
        assert_eq!(Daemon::syncd().name, SYNC_BUS);
        assert_eq!(Daemon::compositor().name, COMPOSITOR_BUS);
        for daemon in [
            Daemon::accountd(),
            Daemon::inferd(),
            Daemon::syncd(),
            Daemon::syncd_settings(),
            Daemon::compositor(),
        ] {
            assert!(required_role(daemon.name).is_some(), "{}", daemon.name);
        }
    }
}
