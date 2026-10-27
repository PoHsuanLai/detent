//! The Privacy page's words: the powers, where each stands, and what a program is called when
//! nothing names it.

use detent_model::{PowerStanding, PrivacyPower, ProgramLook};

/// What the compositor says under a row it fixed: the line the page repeats.
pub(crate) const SET_BY_ADMINISTRATOR: &str = "Set by your administrator";

/// What a program that nothing names is called.
const UNNAMED_PROGRAM: &str = "A program";

/// What a power is called.
pub(crate) fn power_label(power: PrivacyPower) -> &'static str {
    match power {
        PrivacyPower::ScreenRecording => "Screen recording",
        PrivacyPower::KeyboardInput => "Keyboard input",
        PrivacyPower::TypingHelp => "Typing help",
        PrivacyPower::Clipboard => "Clipboard",
        PrivacyPower::ScreenLock => "Screen lock",
        PrivacyPower::Displays => "Displays",
        PrivacyPower::Windows => "Windows",
    }
}

/// The word on the page for where a power stands.
pub(crate) fn standing_label(standing: PowerStanding) -> &'static str {
    match standing {
        PowerStanding::Allowed => "Allowed",
        PowerStanding::Denied => "Denied",
        PowerStanding::Ask => "Ask",
    }
}

/// What a program is called: the name its look has, else a plain word.
pub(crate) fn program_name(look: &ProgramLook) -> String {
    look.name
        .clone()
        .unwrap_or_else(|| UNNAMED_PROGRAM.to_owned())
}

/// The words "Other programs ask the first time.".
pub(crate) const NOT_LISTED_HELP: &str = "Other programs ask the first time.";

/// The words "Nothing has asked yet".
pub(crate) const NOTHING_HAS_ASKED_YET: &str = "Nothing has asked yet";

/// The words "Programs that ask to see your screen or type for you appear here.".
pub(crate) const NO_PROGRAMS_HINT: &str =
    "Programs that ask to see your screen or type for you appear here.";

/// The words "In {place}".
pub(crate) fn in_folder(place: impl std::fmt::Display) -> String {
    format!("In {place}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_powers_are_named_in_plain_words_in_the_pages_order() {
        let labels: Vec<&str> = PrivacyPower::ALL.into_iter().map(power_label).collect();
        assert_eq!(
            labels,
            [
                "Screen recording",
                "Keyboard input",
                "Typing help",
                "Clipboard",
                "Screen lock",
                "Displays",
                "Windows"
            ]
        );
    }

    #[test]
    fn a_standing_has_its_word() {
        let labels: Vec<&str> = PowerStanding::CHOICES
            .into_iter()
            .map(standing_label)
            .collect();
        assert_eq!(labels, ["Allowed", "Denied", "Ask"]);
    }

    #[test]
    fn a_program_nothing_names_is_a_program() {
        let look = |name: Option<&str>| ProgramLook {
            name: name.map(str::to_owned),
            icon: None,
            place: None,
        };
        let cases = [(Some("Grim"), "Grim"), (None, "A program")];
        for (name, want) in cases {
            assert_eq!(program_name(&look(name)), want);
        }
    }
}
