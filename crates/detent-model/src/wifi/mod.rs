//! The Wi-Fi page, as Settings meets it: the radio switch, the network the computer is on and the
//! ones it can see, and joining one. Pure; the network service is the window's (`detent-ui`).

mod model;
mod page;
mod passphrase;
mod step;
mod words;

pub use model::{
    AccessPoint, NetworkLink, NetworkLock, Remembered, SignalBars, SignalStrength, Ssid,
    WifiNetwork, WifiRadio, WifiStatus, WifiWire,
};
pub use page::{WIFI_SLUG, is_wifi, wifi_page};
pub use passphrase::Passphrase;
pub use step::{
    AskStanding, JoinFailure, PasswordAsk, WifiEffect, WifiInput, WifiReady, WifiSheet, WifiState,
    wifi_step,
};
pub use words::{WIFI_BLOCKED, details_rows, missing_words, password_help, password_title};

#[cfg(test)]
mod tests;
