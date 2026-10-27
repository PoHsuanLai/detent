//! The connection a new join adds, as plain settings: what the person gave to join, and the
//! groups of keys the network service takes for it. Pure; the system turns the values into the
//! bus's own.

use super::account::WifiLogin;
use super::join::WifiSecurity;
use super::model::Ssid;
use super::passphrase::Passphrase;
use std::collections::BTreeMap;

/// The group that holds how a network is secured.
const SECURITY_GROUP: &str = "802-11-wireless-security";

/// What the person gave to join.
#[derive(Debug, Clone)]
pub enum JoinSecret {
    /// Nothing: a kept connection is used, or the network is open.
    Kept,
    /// A password.
    Password(Passphrase),
    /// A username and a password.
    Account(WifiLogin),
}

impl JoinSecret {
    /// The password the person gave, if they gave one.
    pub fn of(passphrase: Option<Passphrase>) -> JoinSecret {
        passphrase.map_or(JoinSecret::Kept, JoinSecret::Password)
    }

    /// Whether the person gave something.
    pub fn is_given(&self) -> bool {
        !matches!(self, JoinSecret::Kept)
    }
}

/// Whether a network announces its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiVisibility {
    /// It does.
    Announced,
    /// It does not.
    Hidden,
}

/// One value of a setting group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiSetting {
    /// A word.
    Text(String),
    /// Bytes.
    Bytes(Vec<u8>),
    /// A list of words.
    Texts(Vec<String>),
    /// A switch that is on.
    Enabled,
}

/// A setting group: its keys and their values.
pub type WifiSettingGroup = BTreeMap<String, WifiSetting>;

/// A new connection: its groups by name.
pub type WifiConnection = BTreeMap<String, WifiSettingGroup>;

fn text(value: &str) -> WifiSetting {
    WifiSetting::Text(value.to_owned())
}

fn group<const N: usize>(pairs: [(&str, WifiSetting); N]) -> WifiSettingGroup {
    pairs
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

/// The security group of a network with a password.
fn personal(management: &str, passphrase: &Passphrase) -> WifiSettingGroup {
    group([
        ("key-mgmt", text(management)),
        ("psk", text(&passphrase.clone().into_text())),
    ])
}

/// The 802.1X group of a network that signs in: PEAP with MSCHAPv2, the server's certificate
/// checked against the computer's own authorities.
fn sign_in(account: &WifiLogin) -> WifiSettingGroup {
    group([
        ("eap", WifiSetting::Texts(vec!["peap".to_owned()])),
        ("identity", text(&account.username)),
        ("password", text(&account.passphrase.clone().into_text())),
        ("phase2-auth", text("mschapv2")),
        ("system-ca-certs", WifiSetting::Enabled),
    ])
}

/// What a new connection to `ssid` is made of.
pub fn connection_for(
    ssid: &Ssid,
    security: WifiSecurity,
    secret: &JoinSecret,
    visibility: WifiVisibility,
) -> WifiConnection {
    let mut settings = WifiConnection::new();
    settings.insert(
        "connection".to_owned(),
        group([("type", text("802-11-wireless")), ("id", text(&ssid.0))]),
    );
    let mut wireless = group([("ssid", WifiSetting::Bytes(ssid.0.as_bytes().to_vec()))]);
    if visibility == WifiVisibility::Hidden {
        wireless.insert("hidden".to_owned(), WifiSetting::Enabled);
    }
    match (security, secret) {
        (WifiSecurity::Psk, JoinSecret::Password(passphrase)) => {
            settings.insert(SECURITY_GROUP.to_owned(), personal("wpa-psk", passphrase));
        }
        (WifiSecurity::Sae, JoinSecret::Password(passphrase)) => {
            settings.insert(SECURITY_GROUP.to_owned(), personal("sae", passphrase));
        }
        (WifiSecurity::Eap, JoinSecret::Account(account)) => {
            settings.insert(
                SECURITY_GROUP.to_owned(),
                group([("key-mgmt", text("wpa-eap"))]),
            );
            settings.insert("802-1x".to_owned(), sign_in(account));
        }
        _ => {}
    }
    if settings.contains_key(SECURITY_GROUP) {
        wireless.insert("security".to_owned(), text(SECURITY_GROUP));
    }
    settings.insert("802-11-wireless".to_owned(), wireless);
    settings
}

#[cfg(test)]
mod tests {
    use super::{JoinSecret, WifiVisibility, connection_for};
    use crate::wifi::{Passphrase, Ssid, WifiLogin, WifiSecurity};

    #[test]
    fn a_secured_connection_carries_its_password_and_an_open_one_has_no_security_group() {
        let ssid = Ssid("Home".to_owned());
        let secret = JoinSecret::Password(Passphrase::new("correct horse"));
        let secured = connection_for(&ssid, WifiSecurity::Psk, &secret, WifiVisibility::Announced);
        assert!(secured.contains_key("802-11-wireless-security"));
        let open = connection_for(
            &ssid,
            WifiSecurity::Open,
            &JoinSecret::Kept,
            WifiVisibility::Announced,
        );
        assert!(!open.contains_key("802-11-wireless-security"));
        assert!(open.contains_key("802-11-wireless"));
    }

    #[test]
    fn a_sign_in_carries_its_account_and_a_hidden_network_says_so() {
        let ssid = Ssid("Campus".to_owned());
        let Some(account) = WifiLogin::new("ada", "analytical") else {
            panic!("an account");
        };
        let settings = connection_for(
            &ssid,
            WifiSecurity::Eap,
            &JoinSecret::Account(account),
            WifiVisibility::Hidden,
        );
        let Some(sign_in) = settings.get("802-1x") else {
            panic!("the 802.1X group");
        };
        assert!(sign_in.contains_key("identity") && sign_in.contains_key("password"));
        assert!(settings["802-11-wireless"].contains_key("hidden"));
        let plain = connection_for(
            &ssid,
            WifiSecurity::Open,
            &JoinSecret::Kept,
            WifiVisibility::Announced,
        );
        assert!(!plain["802-11-wireless"].contains_key("hidden"));
    }

    #[test]
    fn a_password_for_a_network_that_signs_in_makes_no_security_group() {
        let ssid = Ssid("Campus".to_owned());
        let secret = JoinSecret::Password(Passphrase::new("correct horse"));
        let settings = connection_for(&ssid, WifiSecurity::Eap, &secret, WifiVisibility::Announced);
        assert!(!settings.contains_key("802-11-wireless-security"));
    }
}
