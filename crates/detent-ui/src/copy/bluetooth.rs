//! The Bluetooth page's words: what the questions ask and how a device's standing and battery read.

use detent_model::{BatteryLevel, BtLink, PairingAsk, PairingCode, PairingQuestion};

/// The title of the question that asks before a device is forgotten.
pub(crate) fn forget_title(name: &str) -> String {
    format!("Forget \u{201c}{name}\u{201d}?")
}

/// The body of that question.
pub(crate) fn forget_body() -> &'static str {
    "This computer will no longer connect to it. To use it again, pair it again."
}

/// The title of a pairing question.
pub(crate) fn pairing_title(question: &PairingQuestion) -> String {
    format!(
        "\u{201c}{}\u{201d} would like to pair with this computer.",
        question.name
    )
}

/// The six-digit number two devices compare while pairing.
fn code(code: PairingCode) -> String {
    format!("{:06}", code.0 % 1_000_000)
}

/// The body of a pairing question.
pub(crate) fn pairing_body(question: &PairingQuestion) -> String {
    match question.ask {
        PairingAsk::Compare(shown) => format!(
            "Confirm that the number {} is shown on \u{201c}{}\u{201d}.",
            code(shown),
            question.name
        ),
        PairingAsk::Allow => "Allow it to pair?".to_owned(),
        PairingAsk::Show(shown) => format!(
            "Type {} on \u{201c}{}\u{201d}, then press Return.",
            code(shown),
            question.name
        ),
        PairingAsk::PinCode => format!("Enter the PIN for \u{201c}{}\u{201d}.", question.name),
        PairingAsk::Passkey => {
            format!("Enter the code shown on \u{201c}{}\u{201d}.", question.name)
        }
    }
}

/// How a device's standing reads.
pub(crate) fn link(link: BtLink) -> &'static str {
    match link {
        BtLink::Disconnected => "Not Connected",
        BtLink::Connecting => "Connecting\u{2026}",
        BtLink::Connected => "Connected",
    }
}

/// The level as the row reads it (`80%`).
pub(crate) fn battery(level: BatteryLevel) -> String {
    format!("{}%", level.0.min(100))
}

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::BtAddress;

    fn question(ask: PairingAsk) -> PairingQuestion {
        PairingQuestion {
            address: BtAddress("AA:BB:CC:DD:EE:04".to_owned()),
            name: "Nearby Pad".to_owned(),
            ask,
        }
    }

    #[test]
    fn a_pairing_question_names_the_device_and_what_to_do() {
        let cases = [
            (
                PairingAsk::Compare(PairingCode(4_217)),
                "Confirm that the number 004217 is shown on \u{201c}Nearby Pad\u{201d}.",
            ),
            (PairingAsk::Allow, "Allow it to pair?"),
            (
                PairingAsk::Show(PairingCode(42)),
                "Type 000042 on \u{201c}Nearby Pad\u{201d}, then press Return.",
            ),
            (
                PairingAsk::PinCode,
                "Enter the PIN for \u{201c}Nearby Pad\u{201d}.",
            ),
            (
                PairingAsk::Passkey,
                "Enter the code shown on \u{201c}Nearby Pad\u{201d}.",
            ),
        ];
        for (ask, want) in cases {
            assert_eq!(pairing_body(&question(ask)), want, "{ask:?}");
        }
        assert_eq!(
            pairing_title(&question(PairingAsk::Allow)),
            "\u{201c}Nearby Pad\u{201d} would like to pair with this computer."
        );
        assert_eq!(forget_title("Zed Buds"), "Forget \u{201c}Zed Buds\u{201d}?");
    }

    #[test]
    fn a_device_and_its_battery_read_in_plain_words() {
        let links = [
            (BtLink::Disconnected, "Not Connected"),
            (BtLink::Connecting, "Connecting\u{2026}"),
            (BtLink::Connected, "Connected"),
        ];
        for (state, want) in links {
            assert_eq!(link(state), want, "{state:?}");
        }
        let batteries = [(80, "80%"), (250, "100%")];
        for (percent, want) in batteries {
            assert_eq!(battery(BatteryLevel(percent)), want, "{percent}");
        }
    }
}
