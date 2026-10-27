//! The modifier keys a corner binding wants held while the pointer rests in the corner.

use std::collections::BTreeSet;

/// One modifier key. The order is the Mac's, which is the order glyphs are written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HotModifier {
    /// Control, written `control`.
    Control,
    /// Option (Alt), written `option`.
    Alt,
    /// Shift, written `shift`.
    Shift,
    /// Command, written `command`.
    Command,
}

impl HotModifier {
    /// Every modifier, in the order glyphs are written.
    pub const ALL: [HotModifier; 4] = [
        HotModifier::Control,
        HotModifier::Alt,
        HotModifier::Shift,
        HotModifier::Command,
    ];

    /// The word the settings file writes.
    pub fn word(self) -> &'static str {
        match self {
            HotModifier::Control => "control",
            HotModifier::Alt => "option",
            HotModifier::Shift => "shift",
            HotModifier::Command => "command",
        }
    }

    /// The modifier a file's word names.
    pub fn of_word(word: &str) -> Option<HotModifier> {
        HotModifier::ALL.into_iter().find(|m| m.word() == word)
    }
}

/// A set of modifier keys. The empty set is the binding that fires with nothing held. Sets order
/// by their members, so the empty one comes first.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HotKeys(BTreeSet<HotModifier>);

impl HotKeys {
    /// The set of `modifiers`.
    pub fn of(modifiers: impl IntoIterator<Item = HotModifier>) -> HotKeys {
        HotKeys(modifiers.into_iter().collect())
    }

    /// The set a file's words name; `None` when any word is not a modifier.
    pub fn of_words<'a>(words: impl IntoIterator<Item = &'a str>) -> Option<HotKeys> {
        words
            .into_iter()
            .map(HotModifier::of_word)
            .collect::<Option<BTreeSet<HotModifier>>>()
            .map(HotKeys)
    }

    /// Whether nothing is held.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Whether `modifier` is in the set.
    pub fn holds(&self, modifier: HotModifier) -> bool {
        self.0.contains(&modifier)
    }

    /// This set with `modifier` taken out if it was in and put in if it was not.
    #[must_use]
    pub fn toggled(mut self, modifier: HotModifier) -> HotKeys {
        if !self.0.remove(&modifier) {
            self.0.insert(modifier);
        }
        self
    }

    /// The words a file writes, in glyph order.
    pub fn words(&self) -> Vec<&'static str> {
        self.0.iter().map(|m| m.word()).collect()
    }

    /// The modifiers held, in glyph order.
    pub fn modifiers(&self) -> Vec<HotModifier> {
        self.0.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{HotKeys, HotModifier};

    #[test]
    fn words_round_trip_in_glyph_order_whatever_order_they_were_written_in() {
        use HotModifier::{Alt, Command, Control, Shift};
        // (words as written, words as read back, the modifiers in order)
        let cases: &[(&[&str], &[&str], &[HotModifier])] = &[
            (&[], &[], &[]),
            (&["option"], &["option"], &[Alt]),
            (
                &["command", "option"],
                &["option", "command"],
                &[Alt, Command],
            ),
            (
                &["shift", "control", "command", "option"],
                &["control", "option", "shift", "command"],
                &[Control, Alt, Shift, Command],
            ),
        ];
        for (written, read, modifiers) in cases {
            let keys = HotKeys::of_words(written.iter().copied()).expect("known words");
            assert_eq!(keys.words(), *read, "{written:?}");
            assert_eq!(keys.modifiers(), *modifiers, "{written:?}");
        }
        assert_eq!(HotKeys::of_words(["hyper"]), None);
    }

    #[test]
    fn toggling_twice_is_the_same_set_and_the_empty_set_sorts_first() {
        let keys = HotKeys::default().toggled(HotModifier::Shift);
        assert!(keys.holds(HotModifier::Shift));
        assert_eq!(keys.clone().toggled(HotModifier::Shift), HotKeys::default());
        let mut sets = [
            HotKeys::of([HotModifier::Command]),
            HotKeys::default(),
            HotKeys::of([HotModifier::Alt]),
        ];
        sets.sort();
        assert_eq!(sets[0], HotKeys::default());
    }
}
