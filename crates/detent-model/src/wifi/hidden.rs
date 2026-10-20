//! Joining a network that does not announce its name: the form the sheet fills and the network it
//! makes once it is complete.

use super::account::WifiLogin;
use super::model::Ssid;
use super::passphrase::Passphrase;
use std::fmt;

/// The longest network name: 32 bytes on the air.
const NAME_LONGEST: usize = 32;

/// How a hidden network is secured, as the sheet's pop-up lists it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HiddenSecurity {
    /// No password.
    #[default]
    Open,
    /// A password (WPA2, also WPA3 in its transition mode).
    Personal,
    /// A password, WPA3 only.
    PersonalWpa3,
    /// A username and a password.
    Enterprise,
}

impl HiddenSecurity {
    /// Every choice, in the order the pop-up lists them.
    pub const ALL: [HiddenSecurity; 4] = [
        HiddenSecurity::Open,
        HiddenSecurity::Personal,
        HiddenSecurity::PersonalWpa3,
        HiddenSecurity::Enterprise,
    ];

    /// The choice as a person reads it.
    pub fn words(self) -> &'static str {
        match self {
            HiddenSecurity::Open => "None",
            HiddenSecurity::Personal => "WPA2/WPA3 Personal",
            HiddenSecurity::PersonalWpa3 => "WPA3 Personal",
            HiddenSecurity::Enterprise => "WPA2/WPA3 Enterprise",
        }
    }

    /// Whether the sheet asks for a password.
    pub fn takes_password(self) -> bool {
        self != HiddenSecurity::Open
    }

    /// Whether the sheet asks for a username.
    pub fn takes_username(self) -> bool {
        self == HiddenSecurity::Enterprise
    }
}

/// What the sheet holds while the person fills it.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct HiddenForm {
    /// The network's name.
    pub name: String,
    /// How it is secured.
    pub security: HiddenSecurity,
    /// The username, for an enterprise network.
    pub username: String,
    /// The password.
    pub password: String,
}

impl fmt::Debug for HiddenForm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HiddenForm")
            .field("name", &self.name)
            .field("security", &self.security)
            .field("password", &"<hidden>")
            .finish_non_exhaustive()
    }
}

/// What a hidden network is joined with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HiddenLogin {
    /// Nothing.
    Open,
    /// A password, WPA2 or WPA3 transition.
    Personal(Passphrase),
    /// A password, WPA3 only.
    PersonalWpa3(Passphrase),
    /// A username and a password.
    Enterprise(WifiLogin),
}

/// A hidden network to join.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiddenNetwork {
    /// Its name.
    pub ssid: Ssid,
    /// What it is joined with.
    pub login: HiddenLogin,
}

impl HiddenForm {
    /// The network the form describes; none while it is incomplete (no name, a name too long for
    /// the air, a password too short to be one, a missing username).
    pub fn network(&self) -> Option<HiddenNetwork> {
        if self.name.is_empty() || self.name.len() > NAME_LONGEST {
            return None;
        }
        let passphrase = Passphrase::new(&self.password);
        let login = match self.security {
            HiddenSecurity::Open => HiddenLogin::Open,
            HiddenSecurity::Personal if passphrase.is_long_enough() => {
                HiddenLogin::Personal(passphrase)
            }
            HiddenSecurity::PersonalWpa3 if passphrase.is_long_enough() => {
                HiddenLogin::PersonalWpa3(passphrase)
            }
            HiddenSecurity::Enterprise => {
                HiddenLogin::Enterprise(WifiLogin::new(&self.username, &self.password)?)
            }
            HiddenSecurity::Personal | HiddenSecurity::PersonalWpa3 => return None,
        };
        Some(HiddenNetwork {
            ssid: Ssid(self.name.clone()),
            login,
        })
    }
}
