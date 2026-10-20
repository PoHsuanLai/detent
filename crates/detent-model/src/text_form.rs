//! A key's value as text, for the callers that have no control to click: a command line, an
//! agent. The words a person reads in the window stay the window's business; text here is the
//! stored form (`off`, `75`, `/home/ada/pics`).

use crate::control::Control;
use crate::range::Number;

/// Why a text is not a value of the control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextFault {
    /// The control has no component, so only the file edits it.
    FileOnly,
    /// A word the control does not offer; these are the words it does.
    NotAWord(Vec<String>),
    /// Not a number.
    NotANumber,
    /// A number outside the range; the range is the first and last value.
    OutOfRange(i64, i64),
}

impl Control {
    /// `text` as a value this control holds.
    pub fn from_text(&self, text: &str) -> Result<toml::Value, TextFault> {
        let word = |words: Vec<String>| {
            if words.iter().any(|w| w == text) {
                Ok(toml::Value::String(text.to_owned()))
            } else {
                Err(TextFault::NotAWord(words))
            }
        };
        match self {
            Control::Readout(only) => word(vec![only.clone()]),
            Control::Toggle(pair) => word(vec![pair.on.clone(), pair.off.clone()]),
            Control::Segmented(words) | Control::Menu(words) => word(words.clone()),
            Control::Slider(range) => {
                let parsed = match range.number {
                    Number::Whole => text.trim().parse().map(toml::Value::Integer).ok(),
                    Number::Fractional => text.trim().parse().map(toml::Value::Float).ok(),
                };
                let value = parsed.ok_or(TextFault::NotANumber)?;
                range
                    .admit(&value)
                    .ok_or(TextFault::OutOfRange(range.min, range.max))
            }
            Control::Text | Control::Colour | Control::Picture => {
                Ok(toml::Value::String(text.to_owned()))
            }
            Control::Rows(_) | Control::FileOnly(_) => Err(TextFault::FileOnly),
        }
    }
}

/// `value` as text: a string as it is, anything else as TOML writes it.
pub fn show_value(value: &toml::Value) -> String {
    match value {
        toml::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{TextFault, show_value};
    use crate::control::{Control, TogglePair};
    use crate::range::{Number, Range};
    use toml::Value;

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|w| (*w).to_owned()).collect()
    }

    fn whole() -> Control {
        Control::Slider(Range {
            min: 32,
            max: 80,
            unit: None,
            number: Number::Whole,
        })
    }

    #[test]
    fn text_becomes_the_value_each_control_holds() {
        let toggle = Control::Toggle(TogglePair {
            on: "on".into(),
            off: "off".into(),
        });
        let segments = Control::Segmented(words(&["light", "dark", "auto"]));
        let cases: Vec<(Control, &str, Result<Value, TextFault>)> = vec![
            (toggle.clone(), "off", Ok(Value::String("off".into()))),
            (
                toggle,
                "maybe",
                Err(TextFault::NotAWord(words(&["on", "off"]))),
            ),
            (segments.clone(), "dark", Ok(Value::String("dark".into()))),
            (
                segments,
                "Dark",
                Err(TextFault::NotAWord(words(&["light", "dark", "auto"]))),
            ),
            (whole(), "48", Ok(Value::Integer(48))),
            (whole(), " 48 ", Ok(Value::Integer(48))),
            (whole(), "31", Err(TextFault::OutOfRange(32, 80))),
            (whole(), "81", Err(TextFault::OutOfRange(32, 80))),
            (whole(), "big", Err(TextFault::NotANumber)),
            (whole(), "4.5", Err(TextFault::NotANumber)),
            (
                Control::Text,
                "/tmp/x y",
                Ok(Value::String("/tmp/x y".into())),
            ),
            (
                Control::FileOnly(crate::control::FileOnly::List),
                "a",
                Err(TextFault::FileOnly),
            ),
        ];
        for (control, text, want) in cases {
            assert_eq!(control.from_text(text), want, "{control:?} {text:?}");
        }
    }

    #[test]
    fn a_fractional_slider_reads_decimals() {
        let control = Control::Slider(Range {
            min: 0,
            max: 10,
            unit: None,
            number: Number::Fractional,
        });
        assert_eq!(control.from_text("2.5"), Ok(Value::Float(2.5)));
        assert_eq!(control.from_text("11"), Err(TextFault::OutOfRange(0, 10)));
    }

    #[test]
    fn values_show_as_their_stored_text() {
        let cases = [
            (Value::String("off".into()), "off"),
            (Value::Integer(75), "75"),
            (Value::Float(1.5), "1.5"),
            (Value::Boolean(true), "true"),
        ];
        for (value, want) in cases {
            assert_eq!(show_value(&value), want);
        }
    }
}
