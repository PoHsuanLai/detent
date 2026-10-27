//! The powers a program may be given.

/// What a program may be allowed to do. The compositor names each power in the path of its rows
/// (`privacy.<slug>.…`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PrivacyPower {
    /// See everything shown on the screen.
    ScreenRecording,
    /// Press keys and buttons for the person.
    KeyboardInput,
    /// See what is typed, to offer suggestions.
    TypingHelp,
    /// Read and replace what was copied.
    Clipboard,
    /// Cover the screen and ask for the password.
    ScreenLock,
    /// Change the displays.
    Displays,
    /// See and manage other programs' windows.
    Windows,
}

/// `(power, the word in a row's path)`, in the order the page lists them.
/// The position of a row is the power's discriminant.
const WORDS: [(PrivacyPower, &str); 7] = [
    (PrivacyPower::ScreenRecording, "screen"),
    (PrivacyPower::KeyboardInput, "keys"),
    (PrivacyPower::TypingHelp, "typing_help"),
    (PrivacyPower::Clipboard, "clipboard"),
    (PrivacyPower::ScreenLock, "lock"),
    (PrivacyPower::Displays, "displays"),
    (PrivacyPower::Windows, "windows"),
];

impl PrivacyPower {
    /// Every power, in the order the page lists them.
    pub const ALL: [PrivacyPower; 7] = [
        WORDS[0].0, WORDS[1].0, WORDS[2].0, WORDS[3].0, WORDS[4].0, WORDS[5].0, WORDS[6].0,
    ];

    /// The word the compositor writes for it in a row's path (`screen`, `keys`, `typing_help`…).
    pub fn word(self) -> &'static str {
        WORDS[self as usize].1
    }

    /// The power a row's path names by `word`; none for a word this page does not know.
    pub(super) fn of_word(word: &str) -> Option<PrivacyPower> {
        WORDS.iter().find(|row| row.1 == word).map(|row| row.0)
    }
}
