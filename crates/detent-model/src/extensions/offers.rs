//! The choices a `Provided` key offers: the usable Extensions of its capability, `standard`
//! first. The files layer finds them; this is only their shape and how a key takes them.

use ds_settings::schema::{STANDARD, WordLabels};
use std::collections::BTreeMap;

/// What a menu of installed things shows for one capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// The stored words, `standard` first.
    pub words: Vec<String>,
    /// The name of each word, as a person reads it.
    pub labels: WordLabels,
}

impl Offer {
    /// Only the built-in choice: what a menu offers while nothing is installed.
    pub fn standard() -> Offer {
        Offer {
            words: vec![STANDARD.to_owned()],
            labels: WordLabels([(STANDARD.to_owned(), "Standard".to_owned())].into()),
        }
    }
}

/// The offer of each capability, by the capability's word (`look`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Offers(BTreeMap<String, Offer>);

impl Offers {
    /// These offers with `offer` for `capability`.
    #[must_use]
    pub fn with(mut self, capability: &str, offer: Offer) -> Offers {
        self.0.insert(capability.to_owned(), offer);
        self
    }

    /// What `capability` offers; only `standard` when nothing was found for it.
    pub fn of(&self, capability: &str) -> Offer {
        self.0
            .get(capability)
            .cloned()
            .unwrap_or_else(Offer::standard)
    }
}

/// The stored word of `value` when it is on none of `words`: a choice whose Extension is gone
/// or turned off, which the menu keeps and shows as unavailable.
pub fn stray_word<'a>(words: &[String], value: &'a toml::Value) -> Option<&'a str> {
    value
        .as_str()
        .filter(|word| !word.is_empty() && !words.iter().any(|offered| offered == word))
}

#[cfg(test)]
mod tests {
    use super::{Offer, Offers, stray_word};
    use ds_settings::schema::WordLabels;
    use toml::Value;

    fn paper() -> Offer {
        Offer {
            words: vec!["standard".to_owned(), "org.example.Paper".to_owned()],
            labels: WordLabels(
                [
                    ("standard".to_owned(), "Standard".to_owned()),
                    ("org.example.Paper".to_owned(), "Paper".to_owned()),
                ]
                .into(),
            ),
        }
    }

    #[test]
    fn a_capability_nothing_was_found_for_offers_only_standard() {
        let offers = Offers::default().with("look", paper());
        assert_eq!(offers.of("look"), paper());
        assert_eq!(offers.of("file_icons"), Offer::standard());
    }

    #[test]
    fn a_stored_word_off_the_menu_is_a_stray_and_the_others_are_not() {
        let words = paper().words;
        let text = |word: &str| Value::String(word.to_owned());
        assert_eq!(stray_word(&words, &text("standard")), None);
        assert_eq!(stray_word(&words, &text("org.example.Paper")), None);
        assert_eq!(
            stray_word(&words, &text("org.example.Gone")),
            Some("org.example.Gone")
        );
        assert_eq!(stray_word(&words, &text("")), None);
        assert_eq!(stray_word(&words, &Value::Integer(3)), None);
    }
}
