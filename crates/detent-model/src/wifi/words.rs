//! The page's words: what is said when there is nothing to show, what the password sheet asks, and
//! the rows of the details sheet.

use super::model::{NetworkLock, SignalBars, SignalStrength, WifiNetwork};
use super::step::{AskKind, AskStanding};
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
pub fn password_title(ssid: &str, kind: AskKind) -> String {
    let wants = match kind {
        AskKind::Password => "a password",
        AskKind::Account => "a username and password",
    };
    format!("The Wi-Fi network \u{201c}{ssid}\u{201d} requires {wants}.")
}

/// The line under the password sheet's title.
pub fn password_help(standing: AskStanding, kind: AskKind) -> &'static str {
    match (standing, kind) {
        (AskStanding::First, AskKind::Password) => "Enter the password to join this network.",
        (AskStanding::First, AskKind::Account) => {
            "Enter your username and password to join this network."
        }
        (AskStanding::Wrong, AskKind::Password) => "The password is incorrect. Try again.",
        (AskStanding::Wrong, AskKind::Account) => {
            "The username or password is incorrect. Try again."
        }
    }
}

/// The title of the sheet that asks before a network is forgotten.
pub fn forget_network_title(ssid: &str) -> String {
    format!("Forget the Wi-Fi network \u{201c}{ssid}\u{201d}?")
}

/// The body of that sheet.
pub fn forget_network_body() -> &'static str {
    "This computer will no longer join it by itself. To use it again, enter its password again."
}

/// The title of the sheet that joins a network that does not announce its name.
pub const OTHER_TITLE: &str = "Find and join a Wi-Fi network.";

/// The line under that sheet's title.
pub const OTHER_HELP: &str = "Enter the name and security type of the network you want to join.";

/// What is said when a join to a network that is not in the list fails: the name, the security
/// or the password may be wrong, and the service does not tell which.
pub const OTHER_FAILED: &str =
    "Couldn\u{2019}t join the network. Check its name, security and password.";

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
            NetworkLock::Enterprise => "Enterprise",
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
