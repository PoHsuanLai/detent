//! The Wi-Fi page's words: what the switch says when the computer has turned Wi-Fi off, what the
//! password and forget questions ask, and the rows of the details.

use detent_model::{
    AskKind, AskStanding, HiddenSecurity, NetworkLock, SignalBars, SignalStrength, WifiNetwork,
    WifiRadio,
};

/// The line under the switch: only a switch the computer's own key or switch turned off has one.
pub(crate) fn radio_help(radio: WifiRadio) -> Option<&'static str> {
    match radio {
        WifiRadio::Blocked => {
            Some("Wi-Fi is switched off on this computer. Turn it on with its key or switch.")
        }
        WifiRadio::Off | WifiRadio::On => None,
    }
}

/// What the password question says it is for.
pub(crate) fn password_title(ssid: &str, kind: AskKind) -> String {
    let wants = match kind {
        AskKind::Password => "a password",
        AskKind::Account => "a username and password",
    };
    format!("The Wi-Fi network \u{201c}{ssid}\u{201d} requires {wants}.")
}

/// The line under the password question's title.
pub(crate) fn password_help(standing: AskStanding, kind: AskKind) -> &'static str {
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

/// The title of the question that asks before a network is forgotten.
pub(crate) fn forget_title(ssid: &str) -> String {
    format!("Forget the Wi-Fi network \u{201c}{ssid}\u{201d}?")
}

/// The body of that question.
pub(crate) fn forget_body() -> &'static str {
    "This computer will no longer join it by itself. To use it again, enter its password again."
}

/// The title of the question that joins a network that does not announce its name.
pub(crate) fn other_title() -> &'static str {
    "Find and join a Wi-Fi network."
}

/// The line under that title.
pub(crate) fn other_help() -> &'static str {
    "Enter the name and security type of the network you want to join."
}

/// What is said when a join to a network that is not in the list fails: the name, the security
/// or the password may be wrong, and the service does not tell which.
pub(crate) fn could_not_join() -> &'static str {
    "Couldn\u{2019}t join the network. Check its name, security and password."
}

/// The signal as a person reads it.
pub(crate) fn signal(bars: SignalBars) -> &'static str {
    match bars {
        SignalBars::Weak => "Weak",
        SignalBars::Fair => "Fair",
        SignalBars::Good => "Good",
    }
}

/// The security as a person reads it.
pub(crate) fn lock(lock: NetworkLock) -> &'static str {
    match lock {
        NetworkLock::Open => "None",
        NetworkLock::Secured => "Password protected",
        NetworkLock::Enterprise => "Enterprise",
    }
}

/// The choice of security for a network that hides its name, as the pop-up lists it.
pub(crate) fn hidden_security(security: HiddenSecurity) -> &'static str {
    match security {
        HiddenSecurity::Open => "None",
        HiddenSecurity::Personal => "WPA2/WPA3 Personal",
        HiddenSecurity::PersonalWpa3 => "WPA3 Personal",
        HiddenSecurity::Enterprise => "WPA2/WPA3 Enterprise",
    }
}

/// The strength in percent, as it reads in the details.
pub(crate) fn percent(strength: SignalStrength) -> String {
    format!("{} percent", strength.0)
}

/// The rows of the details: a label and its value.
pub(crate) fn details_rows(network: &WifiNetwork) -> Vec<(&'static str, String)> {
    vec![
        ("Network Name", network.ssid.0.clone()),
        ("Security", lock(network.lock).to_owned()),
        (
            "Signal",
            format!(
                "{} ({})",
                signal(network.strength.signal()),
                percent(network.strength)
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::{NetworkLink, Remembered, Ssid};

    #[test]
    fn the_switch_has_a_line_only_when_the_computer_turned_it_off() {
        let cases = [
            (WifiRadio::On, None),
            (WifiRadio::Off, None),
            (
                WifiRadio::Blocked,
                Some("Wi-Fi is switched off on this computer. Turn it on with its key or switch."),
            ),
        ];
        for (radio, want) in cases {
            assert_eq!(radio_help(radio), want, "{radio:?}");
        }
    }

    #[test]
    fn the_password_question_names_the_network_and_what_it_wants() {
        let cases = [
            (
                AskKind::Password,
                "The Wi-Fi network \u{201c}Cafe\u{201d} requires a password.",
            ),
            (
                AskKind::Account,
                "The Wi-Fi network \u{201c}Cafe\u{201d} requires a username and password.",
            ),
        ];
        for (kind, want) in cases {
            assert_eq!(password_title("Cafe", kind), want, "{kind:?}");
        }
        assert_eq!(
            forget_title("Cafe"),
            "Forget the Wi-Fi network \u{201c}Cafe\u{201d}?"
        );
    }

    #[test]
    fn the_line_under_the_question_says_whether_the_last_try_was_refused() {
        let cases = [
            (
                AskStanding::First,
                AskKind::Password,
                "Enter the password to join this network.",
            ),
            (
                AskStanding::First,
                AskKind::Account,
                "Enter your username and password to join this network.",
            ),
            (
                AskStanding::Wrong,
                AskKind::Password,
                "The password is incorrect. Try again.",
            ),
            (
                AskStanding::Wrong,
                AskKind::Account,
                "The username or password is incorrect. Try again.",
            ),
        ];
        for (standing, kind, want) in cases {
            assert_eq!(password_help(standing, kind), want, "{standing:?} {kind:?}");
        }
    }

    #[test]
    fn the_security_choices_read_as_the_pop_up_lists_them() {
        let cases = [
            (HiddenSecurity::Open, "None"),
            (HiddenSecurity::Personal, "WPA2/WPA3 Personal"),
            (HiddenSecurity::PersonalWpa3, "WPA3 Personal"),
            (HiddenSecurity::Enterprise, "WPA2/WPA3 Enterprise"),
        ];
        for (choice, want) in cases {
            assert_eq!(hidden_security(choice), want, "{choice:?}");
        }
    }

    #[test]
    fn the_details_rows_name_the_network_its_security_and_its_signal() {
        let office = WifiNetwork {
            ssid: Ssid("Office".to_owned()),
            strength: SignalStrength(90),
            lock: NetworkLock::Secured,
            link: NetworkLink::Joined,
            remembered: Remembered::Yes,
        };
        assert_eq!(
            details_rows(&office),
            [
                ("Network Name", "Office".to_owned()),
                ("Security", "Password protected".to_owned()),
                ("Signal", "Good (90 percent)".to_owned()),
            ]
        );
    }

    #[test]
    fn signals_and_locks_read_in_plain_words() {
        let signals = [
            (SignalBars::Weak, "Weak"),
            (SignalBars::Fair, "Fair"),
            (SignalBars::Good, "Good"),
        ];
        for (bars, want) in signals {
            assert_eq!(signal(bars), want, "{bars:?}");
        }
        let locks = [
            (NetworkLock::Open, "None"),
            (NetworkLock::Secured, "Password protected"),
            (NetworkLock::Enterprise, "Enterprise"),
        ];
        for (mark, want) in locks {
            assert_eq!(lock(mark), want, "{mark:?}");
        }
    }
}
