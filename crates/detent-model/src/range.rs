//! A bounded number: where it sits on its slider and how it reads.

use ds_settings::schema::KeyKind;

/// Whether a bounded key holds whole numbers or fractions, read off its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Number {
    /// Pixels, milliseconds, counts.
    Whole,
    /// A physics constant such as a momentum exponent.
    Fractional,
}

/// A number with a floor and a ceiling and the unit it is shown in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// The smallest value.
    pub min: i64,
    /// The largest value.
    pub max: i64,
    /// `px`, `ms`, `%`, or none.
    pub unit: Option<String>,
    /// Whole or fractional.
    pub number: Number,
}

/// A slider's whole travel, in thousandths (`ds::prelude::Fraction`'s unit).
const FULL: f64 = 1000.0;

impl Range {
    /// The range of a `Bounded` kind whose default is `default`.
    pub fn of(kind: &KeyKind, default: &toml::Value) -> Option<Range> {
        let KeyKind::Bounded { min, max, unit } = kind else {
            return None;
        };
        let number = match default {
            toml::Value::Float(_) => Number::Fractional,
            _ => Number::Whole,
        };
        Some(Range {
            min: *min,
            max: *max,
            unit: unit.clone(),
            number,
        })
    }

    /// `value` as this range's number, when it is one and lies inside the range.
    pub fn admit(&self, value: &toml::Value) -> Option<toml::Value> {
        let inside = |n: f64| n >= self.min as f64 && n <= self.max as f64;
        match (value, self.number) {
            (toml::Value::Integer(n), Number::Whole) if inside(*n as f64) => Some(value.clone()),
            (toml::Value::Integer(n), Number::Fractional) if inside(*n as f64) => {
                Some(toml::Value::Float(*n as f64))
            }
            (toml::Value::Float(n), Number::Fractional) if inside(*n) => Some(value.clone()),
            _ => None,
        }
    }

    fn number_of(value: &toml::Value) -> f64 {
        match value {
            toml::Value::Integer(n) => *n as f64,
            toml::Value::Float(n) => *n,
            _ => 0.0,
        }
    }

    /// Where `value` stands on the slider, in thousandths (0 at the floor).
    pub fn position(&self, value: &toml::Value) -> u16 {
        let span = (self.max - self.min) as f64;
        if span <= 0.0 {
            return 0;
        }
        let at = (Self::number_of(value) - self.min as f64) / span;
        (at.clamp(0.0, 1.0) * FULL).round() as u16
    }

    /// The value at `position` thousandths of the travel.
    pub fn value_at(&self, position: u16) -> toml::Value {
        let span = (self.max - self.min) as f64;
        let raw = self.min as f64 + span * f64::from(position.min(1000)) / FULL;
        match self.number {
            Number::Whole => toml::Value::Integer(raw.round() as i64),
            Number::Fractional => toml::Value::Float((raw * FULL).round() / FULL),
        }
    }

    /// `value` as a person reads it: `48 px`, `0.55`.
    pub fn display(&self, value: &toml::Value) -> String {
        let number = match value {
            toml::Value::Integer(n) => n.to_string(),
            toml::Value::Float(n) => n.to_string(),
            _ => String::new(),
        };
        match &self.unit {
            Some(unit) => format!("{number} {unit}"),
            None => number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Number, Range};
    use toml::Value;

    fn whole() -> Range {
        Range {
            min: 48,
            max: 128,
            unit: Some("px".to_owned()),
            number: Number::Whole,
        }
    }

    fn fractional() -> Range {
        Range {
            min: 0,
            max: 4,
            unit: None,
            number: Number::Fractional,
        }
    }

    #[test]
    fn a_slider_position_maps_to_a_value_and_back() {
        // `(case, range, position, value, as a person reads it)`.
        let cases: &[(&str, Range, u16, Value, &str)] = &[
            ("floor", whole(), 0, Value::Integer(48), "48 px"),
            ("ceiling", whole(), 1000, Value::Integer(128), "128 px"),
            ("middle", whole(), 500, Value::Integer(88), "88 px"),
            ("fraction", fractional(), 250, Value::Float(1.0), "1"),
            ("fraction top", fractional(), 1000, Value::Float(4.0), "4"),
        ];
        for (name, range, position, value, shown) in cases {
            assert_eq!(&range.value_at(*position), value, "{name}: value_at");
            assert_eq!(range.position(value), *position, "{name}: position");
            assert_eq!(&range.display(value), shown, "{name}: display");
        }
    }

    #[test]
    fn a_number_outside_the_range_is_refused() {
        let cases: &[(&str, Range, Value, Option<Value>)] = &[
            (
                "inside",
                whole(),
                Value::Integer(64),
                Some(Value::Integer(64)),
            ),
            ("below", whole(), Value::Integer(10), None),
            ("above", whole(), Value::Integer(129), None),
            ("word", whole(), Value::String("big".to_owned()), None),
            ("float for whole", whole(), Value::Float(64.0), None),
            (
                "integer for fraction",
                fractional(),
                Value::Integer(2),
                Some(Value::Float(2.0)),
            ),
        ];
        for (name, range, value, want) in cases {
            assert_eq!(&range.admit(value), want, "{name}");
        }
    }
}
