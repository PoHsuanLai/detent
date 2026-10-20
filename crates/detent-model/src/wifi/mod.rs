//! The Wi-Fi page, as Settings meets it: the radio switch, the network the computer is on and the
//! ones it can see, and joining one. Pure; the network service is the window's (`detent-ui`).

mod account;
mod hidden;
mod model;
mod page;
mod passphrase;
mod step;
mod words;

pub use account::Account;
pub use hidden::{HiddenForm, HiddenLogin, HiddenNetwork, HiddenSecurity};
pub use model::{
    AccessPoint, NetworkLink, NetworkLock, Remembered, SignalBars, SignalStrength, Ssid,
    WifiNetwork, WifiRadio, WifiStatus, WifiWire,
};
pub use page::{WIFI_SLUG, is_wifi, wifi_page};
pub use passphrase::Passphrase;
pub use step::{
    AskKind, AskStanding, JoinFailure, PasswordAsk, WifiEffect, WifiInput, WifiReady, WifiSheet,
    WifiState, wifi_step,
};
pub use words::{
    OTHER_FAILED, OTHER_HELP, OTHER_TITLE, WIFI_BLOCKED, details_rows, forget_network_body,
    forget_network_title, missing_words, password_help, password_title,
};

#[cfg(test)]
mod tests;
