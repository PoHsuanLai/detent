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

/// Words that mean "not doing it": when a two-word enum has one of them, the other is on.
const OFF_WORDS: &[&str] = &["off", "hide", "never", "none", "nothing"];

impl TogglePair {
    /// The pair of `variants`: the first word is on, unless it is an off word and the second is
    /// not.
    pub fn of(variants: &[String; 2]) -> TogglePair {
        let [first, second] = variants;
        let first_is_off = OFF_WORDS.contains(&first.as_str());
        let second_is_off = OFF_WORDS.contains(&second.as_str());
        if first_is_off && !second_is_off {
            TogglePair {
                on: second.clone(),
                off: first.clone(),
            }
        } else {
            TogglePair {
                on: first.clone(),
                off: second.clone(),
            }
        }
    }
}

/// A kind quire has no component for: shown read-only with an "Edit in file" note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOnly {
    /// A captured key chord: quire has no shortcut-capture field.
    Shortcut,
    /// A list of values: quire has no rows editor.
    List,
}

/// What draws a key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Control {
    /// A one-word enum: read-only text.
    Readout(String),
    /// A two-word enum: a switch.
    Toggle(TogglePair),
    /// Three to five words: a segmented control.
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
            KeyKind::Toggle { variants } => Control::Toggle(TogglePair::of(variants)),
            KeyKind::Segmented { variants } => Control::Segmented(variants.clone()),
            KeyKind::Menu { variants } => Control::Menu(variants.clone()),
            KeyKind::Bounded { .. } => {
                Range::of(&spec.kind, &spec.default).map_or(Control::Text, Control::Slider)
            }
            KeyKind::Text => Control::Text,
            KeyKind::Colour => Control::Colour,
            KeyKind::Shortcut => Control::FileOnly(FileOnly::Shortcut),
            KeyKind::List(_) => Control::FileOnly(FileOnly::List),
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
    use ds_settings::schema::{Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section};
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
        }
    }

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|w| (*w).to_owned()).collect()
    }

    fn pair(a: &str, b: &str) -> TogglePair {
        TogglePair::of(&[a.to_owned(), b.to_owned()])
    }

    #[test]
    fn a_toggle_is_on_at_the_word_that_is_not_off() {
        let cases: &[(&str, &str, &str, &str)] = &[
            ("on", "off", "on", "off"),
            ("off", "on", "on", "off"),
            ("natural", "traditional", "natural", "traditional"),
            ("off", "hardware", "hardware", "off"),
            ("memory", "off", "memory", "off"),
            ("cycle", "nothing", "cycle", "nothing"),
            ("screen_off", "never", "screen_off", "never"),
            ("hide", "show", "show", "hide"),
        ];
        for (a, b, on, off) in cases {
            let got = pair(a, b);
            assert_eq!((got.on.as_str(), got.off.as_str()), (*on, *off), "{a}/{b}");
        }
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
                Control::Toggle(pair("on", "off")),
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
        let toggle = Control::Toggle(pair("on", "off"));
        let text = |s: &str| Value::String(s.to_owned());
        assert_eq!(toggle.admit(&text("on")), Some(text("on")));
        assert_eq!(toggle.admit(&text("bounceyy")), None);
        assert_eq!(toggle.admit(&Value::Integer(1)), None);
        assert_eq!(Control::Text.admit(&text("hello")), Some(text("hello")));
        assert_eq!(Control::Text.admit(&Value::Integer(1)), None);
    }
}
