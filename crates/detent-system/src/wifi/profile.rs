//! A new connection's settings as the network service takes them: the model's plain groups turned
//! into the bus's values.

use airwaves_nm::ConnectionToAdd;
use detent_model::{
    JoinSecret, Ssid, WifiConnection, WifiSecurity, WifiSetting, WifiVisibility, connection_for,
};
use std::collections::HashMap;
use zbus::zvariant::Value;

fn value_of(setting: WifiSetting) -> Value<'static> {
    match setting {
        WifiSetting::Text(text) => Value::from(text),
        WifiSetting::Bytes(bytes) => Value::from(bytes),
        WifiSetting::Texts(texts) => Value::from(texts),
        WifiSetting::Enabled => Value::from(true),
    }
}

fn wire_of(connection: WifiConnection) -> ConnectionToAdd {
    connection
        .into_iter()
        .map(|(name, group)| {
            let keys: HashMap<String, Value<'static>> = group
                .into_iter()
                .map(|(key, setting)| (key, value_of(setting)))
                .collect();
            (name, keys)
        })
        .collect()
}

/// What a new connection to `ssid` is made of, as the service takes it.
pub fn connection_of(
    ssid: &Ssid,
    security: WifiSecurity,
    secret: &JoinSecret,
    visibility: WifiVisibility,
) -> ConnectionToAdd {
    wire_of(connection_for(ssid, security, secret, visibility))
}

#[cfg(test)]
mod tests {
    use super::connection_of;
    use detent_model::{JoinSecret, Passphrase, Ssid, WifiSecurity, WifiVisibility};

    #[test]
    fn the_groups_reach_the_wire_under_their_names() {
        let secret = JoinSecret::Password(Passphrase::new("correct horse"));
        let wire = connection_of(
            &Ssid("Home".to_owned()),
            WifiSecurity::Psk,
            &secret,
            WifiVisibility::Hidden,
        );
        for group in ["connection", "802-11-wireless", "802-11-wireless-security"] {
            assert!(wire.contains_key(group), "{group}");
        }
        assert!(wire["802-11-wireless"].contains_key("hidden"));
    }
}
