//! The page's words: what is said when there is nothing to show, what the password sheet asks, and
//! the rows of the details sheet.

use super::model::{NetworkLock, SignalBars, SignalStrength, WifiNetwork};
use super::step::AskStanding;
use crate::missing::Missing;

/// The line under the switch when the computer's own key or switch has turned Wi-Fi off.
pub const WIFI_BLOCKED: &str =
    "Wi-Fi is switched off on this computer. Turn it on with its key or switch.";

/// What the page says when there is nothing to show.
pub fn missing_words(missing: Missing) -> &'static str {
    match missing {
        Missing::NoService => "Wi-Fi isn\u{2019}t available right now.",
        Missing::NoDevice => "This computer has no Wi-Fi.",
        Missing::Failed => "Wi-Fi didn\u{2019}t answer. Try again in a moment.",
    }
}

/// The password sheet's title.
pub fn password_title(ssid: &str) -> String {
    format!("The Wi-Fi network \u{201c}{ssid}\u{201d} requires a password.")
}

/// The line under the password sheet's title.
pub fn password_help(standing: AskStanding) -> &'static str {
    match standing {
        AskStanding::First => "Enter the password to join this network.",
        AskStanding::Wrong => "The password is incorrect. Try again.",
    }
}

impl SignalBars {
    /// The signal as a person reads it.
    pub fn words(self) -> &'static str {
        match self {
            SignalBars::Weak => "Weak",
            SignalBars::Fair => "Fair",
            SignalBars::Good => "Good",
        }
    }
}

impl NetworkLock {
    /// The security as a person reads it.
    pub fn words(self) -> &'static str {
        match self {
            NetworkLock::Open => "None",
            NetworkLock::Secured => "Password protected",
        }
    }
}

impl SignalStrength {
    /// The strength in percent, as it reads in the details.
    pub fn percent_words(self) -> String {
        format!("{} percent", self.0)
    }
}

/// The rows of the details sheet: a label and its value.
pub fn details_rows(network: &WifiNetwork) -> Vec<(&'static str, String)> {
    vec![
        ("Network Name", network.ssid.0.clone()),
        ("Security", network.lock.words().to_owned()),
        (
            "Signal",
            format!(
                "{} ({})",
                network.strength.signal().words(),
                network.strength.percent_words()
            ),
        ),
    ]
}
