//! The connection a new join adds: how a network is secured, what the person gave to join it, and
//! the setting groups the network service takes for that.

use super::nm::Settings;
use detent_model::{Passphrase, Ssid, WifiLogin};
use std::collections::HashMap;
use zbus::zvariant::Value;

/// The group that holds how a network is secured.
const SECURITY_GROUP: &str = "802-11-wireless-security";

/// How a network is secured, from its access point's flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Security {
    Open,
    Psk,
    Sae,
    /// A sign-in with an account.
    Eap,
    /// An old scheme this page does not do.
    Unsupported,
}

/// The security `flags`, `wpa` and `rsn` of an access point describe.
pub(super) fn security_of(flags: u32, wpa: u32, rsn: u32) -> Security {
    const PRIVACY: u32 = 0x1;
    const PSK: u32 = 0x100;
    const ENTERPRISE: u32 = 0x200;
    const SAE: u32 = 0x400;
    let keys = wpa | rsn;
    if keys & ENTERPRISE != 0 && keys & (PSK | SAE) == 0 {
        Security::Eap
    } else if keys & PSK != 0 {
        Security::Psk
    } else if keys & SAE != 0 {
        Security::Sae
    } else if flags & PRIVACY != 0 || keys != 0 {
        Security::Unsupported
    } else {
        Security::Open
    }
}

/// What the person gave to join.
#[derive(Debug, Clone)]
pub(super) enum Secret {
    /// Nothing: a kept connection is used, or the network is open.
    Kept,
    /// A password.
    Password(Passphrase),
    /// A username and a password.
    Account(WifiLogin),
}

impl Secret {
    /// The password the person gave, if they gave one.
    pub(super) fn of(passphrase: Option<Passphrase>) -> Secret {
        passphrase.map_or(Secret::Kept, Secret::Password)
    }

    /// Whether the person gave something.
    pub(super) fn is_given(&self) -> bool {
        !matches!(self, Secret::Kept)
    }
}

/// Whether a network announces its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Visibility {
    Announced,
    Hidden,
}

fn text(value: &str) -> Value<'static> {
    Value::from(value.to_owned())
}

fn group<const N: usize>(pairs: [(&str, Value<'static>); N]) -> HashMap<String, Value<'static>> {
    pairs
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect()
}

/// The security group of a network with a password.
fn personal(management: &str, passphrase: &Passphrase) -> HashMap<String, Value<'static>> {
    group([
        ("key-mgmt", text(management)),
        ("psk", text(&passphrase.clone().into_text())),
    ])
}

/// The 802.1X group of a network that signs in: PEAP with MSCHAPv2, the server's certificate
/// checked against the computer's own authorities.
fn sign_in(account: &WifiLogin) -> HashMap<String, Value<'static>> {
    group([
        ("eap", Value::from(vec!["peap".to_owned()])),
        ("identity", text(&account.username)),
        ("password", text(&account.passphrase.clone().into_text())),
        ("phase2-auth", text("mschapv2")),
        ("system-ca-certs", Value::from(true)),
    ])
}

/// What a new connection to `ssid` is made of.
pub(super) fn settings_for(
    ssid: &Ssid,
    security: Security,
    secret: &Secret,
    visibility: Visibility,
) -> Settings {
    let mut settings = Settings::new();
    settings.insert(
        "connection".to_owned(),
        group([("type", text("802-11-wireless")), ("id", text(&ssid.0))]),
    );
    let mut wireless = group([("ssid", Value::from(ssid.0.as_bytes().to_vec()))]);
    if visibility == Visibility::Hidden {
        wireless.insert("hidden".to_owned(), Value::from(true));
    }
    match (security, secret) {
        (Security::Psk, Secret::Password(passphrase)) => {
            settings.insert(SECURITY_GROUP.to_owned(), personal("wpa-psk", passphrase));
        }
        (Security::Sae, Secret::Password(passphrase)) => {
            settings.insert(SECURITY_GROUP.to_owned(), personal("sae", passphrase));
        }
        (Security::Eap, Secret::Account(account)) => {
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
    use super::{Secret, Security, Visibility, security_of, settings_for};
    use detent_model::{Passphrase, Ssid, WifiLogin};

    #[test]
    fn flags_say_how_a_network_is_secured() {
        const CASES: &[(&str, u32, u32, u32, Security)] = &[
            ("open", 0, 0, 0, Security::Open),
            ("wpa2 personal", 1, 0, 0x108, Security::Psk),
            ("wpa3", 1, 0, 0x400, Security::Sae),
            ("wpa2 and wpa3", 1, 0, 0x500, Security::Psk),
            ("enterprise", 1, 0, 0x200, Security::Eap),
            ("wep", 1, 0, 0, Security::Unsupported),
        ];
        for (case, flags, wpa, rsn, want) in CASES {
            assert_eq!(security_of(*flags, *wpa, *rsn), *want, "{case}");
        }
    }

    #[test]
    fn a_secured_connection_carries_its_password_and_an_open_one_has_no_security_group() {
        let ssid = Ssid("Home".to_owned());
        let secret = Secret::Password(Passphrase::new("correct horse"));
        let secured = settings_for(&ssid, Security::Psk, &secret, Visibility::Announced);
        assert!(secured.contains_key("802-11-wireless-security"));
        let open = settings_for(&ssid, Security::Open, &Secret::Kept, Visibility::Announced);
        assert!(!open.contains_key("802-11-wireless-security"));
        assert!(open.contains_key("802-11-wireless"));
    }

    #[test]
    fn a_sign_in_carries_its_account_and_a_hidden_network_says_so() {
        let ssid = Ssid("Campus".to_owned());
        let account = WifiLogin::new("ada", "analytical").expect("an account");
        let settings = settings_for(
            &ssid,
            Security::Eap,
            &Secret::Account(account),
            Visibility::Hidden,
        );
        let sign_in = settings.get("802-1x").expect("the 802.1X group");
        assert!(sign_in.contains_key("identity") && sign_in.contains_key("password"));
        let wireless = settings.get("802-11-wireless").expect("the wireless group");
        assert!(wireless.contains_key("hidden"));
        let plain = settings_for(&ssid, Security::Open, &Secret::Kept, Visibility::Announced);
        assert!(!plain["802-11-wireless"].contains_key("hidden"));
    }

    #[test]
    fn a_password_for_a_network_that_signs_in_makes_no_security_group() {
        let ssid = Ssid("Campus".to_owned());
        let secret = Secret::Password(Passphrase::new("correct horse"));
        let settings = settings_for(&ssid, Security::Eap, &secret, Visibility::Announced);
        assert!(!settings.contains_key("802-11-wireless-security"));
    }
}
