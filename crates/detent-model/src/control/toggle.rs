//! The two words of an on/off enum and which one is on.

/// The two words of an on/off enum, which one is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TogglePair {
    /// The word a switched-on toggle stores.
    pub on: String,
    /// The word a switched-off toggle stores.
    pub off: String,
}

/// Words that mean "not doing it": when a two-word enum has exactly one of them, the other is on.
const OFF_WORDS: &[&str] = &["off", "hide", "never", "none", "nothing"];

/// Pairs that are a setting and its absence without an off word, written `(on, off)`.
/// `reduced`/`standard` is macOS's "Reduce motion": on means reduced; `reduced`/`system` is
/// "Reduce transparency" (appearance.transparency).
const ON_OFF_PAIRS: &[(&str, &str)] = &[
    ("natural", "traditional"),
    ("reduced", "standard"),
    ("reduced", "system"),
    ("enabled", "disabled"),
    ("yes", "no"),
];

impl TogglePair {
    /// The pair `variants` make when they are on and off, else `None`: a choice between two
    /// things (`ring`/`bar`, `bottom`/`left`) is not a switch. Two words are on and off when
    /// exactly one is an off word (the other is on), or they are a row of [`ON_OFF_PAIRS`], in
    /// either order.
    pub fn of(variants: &[String; 2]) -> Option<TogglePair> {
        let [first, second] = variants;
        let is_off = |word: &String| OFF_WORDS.contains(&word.as_str());
        let pair = |on: &String, off: &String| TogglePair {
            on: on.clone(),
            off: off.clone(),
        };
        match (is_off(first), is_off(second)) {
            (false, true) => return Some(pair(first, second)),
            (true, false) => return Some(pair(second, first)),
            (true, true) => return None,
            (false, false) => {}
        }
        ON_OFF_PAIRS.iter().find_map(|(on, off)| {
            let words = (first.as_str(), second.as_str());
            match words {
                (a, b) if (a, b) == (*on, *off) => Some(pair(first, second)),
                (a, b) if (a, b) == (*off, *on) => Some(pair(second, first)),
                _ => None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::TogglePair;

    fn pair(a: &str, b: &str) -> Option<TogglePair> {
        TogglePair::of(&[a.to_owned(), b.to_owned()])
    }

    fn switch(on: &str, off: &str) -> Option<TogglePair> {
        Some(TogglePair {
            on: on.to_owned(),
            off: off.to_owned(),
        })
    }

    #[test]
    fn two_words_are_a_switch_only_when_they_are_on_and_off() {
        // (first word, second word, the switch they make: on then off; None is segments)
        type Case = (
            &'static str,
            &'static str,
            Option<(&'static str, &'static str)>,
        );
        let cases: &[Case] = &[
            // an off word: the other is on, in either order
            ("on", "off", Some(("on", "off"))),
            ("off", "on", Some(("on", "off"))),
            ("show", "hide", Some(("show", "hide"))),
            ("hide", "show", Some(("show", "hide"))),
            ("off", "hardware", Some(("hardware", "off"))),
            ("off", "ecb", Some(("ecb", "off"))),
            ("memory", "off", Some(("memory", "off"))),
            ("auto", "off", Some(("auto", "off"))),
            ("apple_mapping", "off", Some(("apple_mapping", "off"))),
            ("cycle", "nothing", Some(("cycle", "nothing"))),
            ("screen_off", "never", Some(("screen_off", "never"))),
            (
                "off",
                "when_monitor_is_output",
                Some(("when_monitor_is_output", "off")),
            ),
            // the table of pairs, in either order
            ("natural", "traditional", Some(("natural", "traditional"))),
            ("traditional", "natural", Some(("natural", "traditional"))),
            ("standard", "reduced", Some(("reduced", "standard"))),
            // appearance.transparency: reduced is on
            ("system", "reduced", Some(("reduced", "system"))),
            ("reduced", "system", Some(("reduced", "system"))),
            ("reduced", "standard", Some(("reduced", "standard"))),
            ("disabled", "enabled", Some(("enabled", "disabled"))),
            ("yes", "no", Some(("yes", "no"))),
            // two off words are not a pair
            ("none", "nothing", None),
            // a choice between two things
            ("ring", "bar", None),
            ("bottom", "left", None),
            ("top_right", "bottom_right", None),
            ("sheet", "popover", None),
            ("system", "editorial", None),
            ("auto", "manual", None),
            ("h24", "h12", None),
            ("monday", "sunday", None),
            ("when_scrolling", "always", None),
            ("toast", "inverse", None),
            ("after_cooldown", "once_per_gesture", None),
            ("toolkit", "ours", None),
        ];
        for (a, b, want) in cases {
            let want = want.and_then(|(on, off)| switch(on, off));
            assert_eq!(pair(a, b), want, "{a}/{b}");
        }
    }
}
