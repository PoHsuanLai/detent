//! The page's words: what is said when there is nothing to show, and what the sheets ask.

use super::model::{BtLink, PairingAsk, PairingQuestion};
use crate::missing::Missing;

/// What the page says when there is nothing to show.
pub fn missing_words(missing: Missing) -> &'static str {
    match missing {
        Missing::NoService => "Bluetooth isn\u{2019}t available right now.",
        Missing::NoDevice => "This computer has no Bluetooth.",
        Missing::Failed => "Bluetooth didn\u{2019}t answer. Try again in a moment.",
    }
}

/// The title of the sheet that asks before a device is forgotten.
pub fn forget_title(name: &str) -> String {
    format!("Forget \u{201c}{name}\u{201d}?")
}

/// The body of that sheet.
pub fn forget_body() -> &'static str {
    "This computer will no longer connect to it. To use it again, pair it again."
}

/// The title of a pairing sheet.
pub fn pairing_title(question: &PairingQuestion) -> String {
    format!(
        "\u{201c}{}\u{201d} would like to pair with this computer.",
        question.name
    )
}

/// The body of a pairing sheet.
pub fn pairing_body(question: &PairingQuestion) -> String {
    match question.ask {
        PairingAsk::Compare(code) => format!(
            "Confirm that the number {code} is shown on \u{201c}{}\u{201d}.",
            question.name
        ),
        PairingAsk::Allow => "Allow it to pair?".to_owned(),
        PairingAsk::Show(code) => format!(
            "Type {code} on \u{201c}{}\u{201d}, then press Return.",
            question.name
        ),
        PairingAsk::PinCode => format!("Enter the PIN for \u{201c}{}\u{201d}.", question.name),
        PairingAsk::Passkey => {
            format!("Enter the code shown on \u{201c}{}\u{201d}.", question.name)
        }
    }
}

impl BtLink {
    /// How a device's standing reads.
    pub fn words(self) -> &'static str {
        match self {
            BtLink::Disconnected => "Not Connected",
            BtLink::Connecting => "Connecting\u{2026}",
            BtLink::Connected => "Connected",
        }
    }
}
