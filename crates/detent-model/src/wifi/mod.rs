//! The Wi-Fi page, as Settings meets it: the radio switch, the network the computer is on and the
//! ones it can see, and joining one. Pure; the network service is the window's (`detent-ui`).

mod account;
mod hidden;
mod join;
mod model;
mod page;
mod passphrase;
mod profile;
mod step;

pub use account::WifiLogin;
pub use hidden::{HiddenForm, HiddenLogin, HiddenNetwork, HiddenSecurity};
pub use join::{JoinVerdict, WifiSecurity, join_verdict, wifi_link_of, wifi_security_of};
pub use model::{
    AccessPoint, NetworkLink, SignalBars, SignalStrength, Ssid, WifiNetwork, WifiRadio, WifiStatus,
    WifiWire,
};
pub use page::{WIFI_SLUG, wifi_page};
pub use passphrase::Passphrase;
pub use profile::{
    JoinSecret, WifiConnection, WifiSetting, WifiSettingGroup, WifiVisibility, connection_for,
};
pub use step::{
    AskKind, AskStanding, JoinFailure, PasswordAsk, WifiAsking, WifiEffect, WifiInput, WifiReady,
    WifiState, wifi_step,
};

#[cfg(test)]
mod tests;
