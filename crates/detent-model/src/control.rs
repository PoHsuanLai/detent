//! The one control that draws a key (design/22 section 9.1's fixed table), and which values it
//! accepts.

use crate::range::Range;
use ds_settings::schema::{KeyKind, KeySpec};

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
/// `reduced`/`standard` is macOS's "Reduce motion": on means reduced.
const ON_OFF_PAIRS: &[(&str, &str)] = &[
    ("natural", "traditional"),
    ("reduced", "standard"),
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

/// A kind quire has no component for: shown read-only with an "Edit in file" note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOnly {
    /// A captured key chord: quire has no shortcut-capture field.
    Shortcut,
    /// A list of values or of tables (rows): quire has no rows editor.
    List,
    /// An action the owning program runs through its live settings module (a button): detent
    /// has no live client yet, so the row is read-only until the Accounts pane's lane wires one.
    Action,
}

/// What draws a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    /// A one-word enum: read-only text.
    Readout(String),
    /// A two-word enum that is on and off: a switch.
    Toggle(TogglePair),
    /// Three to five words, or two that are not on and off: a segmented control.
    Segmented(Vec<String>),
    /// More: a pop-up button.
    Menu(Vec<String>),
    /// A bounded number: a slider.
    Slider(Range),
    /// Free text: a text field.
    Text,
    /// A colour: the swatch row.
    Colour,
    /// No component: read-only, edited in the file.
    FileOnly(FileOnly),
}

impl Control {
    /// The control for `spec`'s kind. Total: every [`KeyKind`] arm is written, so a new kind is
    /// a compile error here.
    pub fn of(spec: &KeySpec) -> Control {
        match &spec.kind {
            KeyKind::Fixed { variant } => Control::Readout(variant.clone()),
            KeyKind::Toggle { variants } => TogglePair::of(variants)
                .map_or_else(|| Control::Segmented(variants.to_vec()), Control::Toggle),
            KeyKind::Segmented { variants } => Control::Segmented(variants.clone()),
            KeyKind::Menu { variants } => Control::Menu(variants.clone()),
            KeyKind::Bounded { .. } => {
                Range::of(&spec.kind, &spec.default).map_or(Control::Text, Control::Slider)
            }
            KeyKind::Text => Control::Text,
            KeyKind::Colour => Control::Colour,
            KeyKind::Shortcut => Control::FileOnly(FileOnly::Shortcut),
            KeyKind::List(_) | KeyKind::Rows { .. } => Control::FileOnly(FileOnly::List),
            KeyKind::Live { .. } => Control::FileOnly(FileOnly::Action),
        }
    }

    /// `value` if this control can hold it (a number in range, a word of its set, text), else
    /// `None`: the lenient loader then uses the key's default.
    pub fn admit(&self, value: &toml::Value) -> Option<toml::Value> {
        let word_of = |words: &[String]| {
            value
                .as_str()
                .filter(|word| words.iter().any(|w| w == word))
                .map(|_| value.clone())
        };
        match self {
            Control::Readout(word) => word_of(std::slice::from_ref(word)),
            Control::Toggle(pair) => word_of(&[pair.on.clone(), pair.off.clone()]),
            Control::Segmented(words) | Control::Menu(words) => word_of(words),
            Control::Slider(range) => range.admit(value),
            Control::Text | Control::Colour => value.is_str().then(|| value.clone()),
            Control::FileOnly(_) => Some(value.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Control, FileOnly, TogglePair};
    use ds_settings::schema::{
        ActionLabel, ActionWeight, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, LiveAction,
        Page, Section,
    };
    use toml::Value;

    fn spec(kind: KeyKind, default: Value) -> KeySpec {
        KeySpec {
            path: KeyPath("x.y".to_owned()),
            kind,
            default,
            label: Label("Y".to_owned()),
            help: Help(String::new()),
            page: Page::Appearance,
            section: Section(String::new()),
            exposure: Exposure::Basic,
            labels: Default::default(),
            agent: Default::default(),
        }
    }

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|w| (*w).to_owned()).collect()
    }

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
            ("cosmic", "shell", None),
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

    #[test]
    fn a_two_word_key_is_drawn_as_the_rule_says() {
        let two = |a: &str, b: &str| {
            Control::of(&spec(
                KeyKind::Toggle {
                    variants: [a.to_owned(), b.to_owned()],
                },
                Value::String(a.to_owned()),
            ))
        };
        assert_eq!(
            two("on", "off"),
            Control::Toggle(pair("on", "off").unwrap())
        );
        assert_eq!(
            two("ring", "bar"),
            Control::Segmented(words(&["ring", "bar"]))
        );
    }

    #[test]
    fn every_key_kind_has_its_one_control() {
        let cases: Vec<(&str, KeyKind, Value, Control)> = vec![
            (
                "fixed",
                KeyKind::Fixed {
                    variant: "sheet".to_owned(),
                },
                Value::String("sheet".to_owned()),
                Control::Readout("sheet".to_owned()),
            ),
            (
                "toggle",
                KeyKind::Toggle {
                    variants: ["on".to_owned(), "off".to_owned()],
                },
                Value::String("on".to_owned()),
                Control::Toggle(pair("on", "off").unwrap()),
            ),
            (
                "segmented",
                KeyKind::Segmented {
                    variants: words(&["a", "b", "c"]),
                },
                Value::String("a".to_owned()),
                Control::Segmented(words(&["a", "b", "c"])),
            ),
            (
                "menu",
                KeyKind::Menu {
                    variants: words(&["a", "b", "c", "d", "e", "f"]),
                },
                Value::String("a".to_owned()),
                Control::Menu(words(&["a", "b", "c", "d", "e", "f"])),
            ),
            (
                "text",
                KeyKind::Text,
                Value::String(String::new()),
                Control::Text,
            ),
            (
                "colour",
                KeyKind::Colour,
                Value::String("#fff".to_owned()),
                Control::Colour,
            ),
            (
                "shortcut",
                KeyKind::Shortcut,
                Value::String("ctrl+a".to_owned()),
                Control::FileOnly(FileOnly::Shortcut),
            ),
            (
                "list",
                KeyKind::List(Box::new(KeyKind::Text)),
                Value::Array(Vec::new()),
                Control::FileOnly(FileOnly::List),
            ),
            (
                "live action",
                KeyKind::Live {
                    action: LiveAction {
                        label: ActionLabel("Remove".to_owned()),
                        weight: ActionWeight::Destructive,
                    },
                },
                Value::String(String::new()),
                Control::FileOnly(FileOnly::Action),
            ),
        ];
        for (name, kind, default, want) in cases {
            assert_eq!(Control::of(&spec(kind, default)), want, "{name}");
        }
    }

    #[test]
    fn a_bounded_key_is_a_slider_with_its_range() {
        let kind = KeyKind::Bounded {
            min: 1,
            max: 9,
            unit: Some("px".to_owned()),
        };
        let control = Control::of(&spec(kind, Value::Integer(3)));
        assert!(matches!(control, Control::Slider(range) if range.max == 9));
    }

    #[test]
    fn a_control_admits_only_its_own_values() {
        let toggle = Control::Toggle(pair("on", "off").unwrap());
        let text = |s: &str| Value::String(s.to_owned());
        assert_eq!(toggle.admit(&text("on")), Some(text("on")));
        assert_eq!(toggle.admit(&text("bounceyy")), None);
        assert_eq!(toggle.admit(&Value::Integer(1)), None);
        assert_eq!(Control::Text.admit(&text("hello")), Some(text("hello")));
        assert_eq!(Control::Text.admit(&Value::Integer(1)), None);
    }
}
