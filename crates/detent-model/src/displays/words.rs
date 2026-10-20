//! The page's words.

use super::arrange::ArrangeSide;
use super::model::{DisplayRotation, PixelSize, RefreshMilliHz};
use crate::missing::Missing;

/// What the page says when there is nothing to show.
pub fn missing_words(missing: Missing) -> &'static str {
    match missing {
        Missing::NoService => "Displays aren\u{2019}t available right now.",
        Missing::NoDevice => "No display is connected.",
        Missing::Failed => "The desktop didn\u{2019}t answer. Try again in a moment.",
    }
}

/// A refresh rate as a person reads it: "60 Hertz", "59.94 Hertz".
pub fn hertz_words(rate: RefreshMilliHz) -> String {
    let (whole, thousandths) = (rate.0 / 1000, rate.0 % 1000);
    // Two decimals, a trailing zero dropped: 59940 is 59.94, 143981 is 143.98.
    let hundredths = (thousandths + 5) / 10;
    let (whole, hundredths) = if hundredths == 100 {
        (whole + 1, 0)
    } else {
        (whole, hundredths)
    };
    match hundredths {
        0 => format!("{whole} Hertz"),
        h if h % 10 == 0 => format!("{whole}.{} Hertz", h / 10),
        h => format!("{whole}.{h:02} Hertz"),
    }
}

/// A size as a person reads it: "3024 \u{d7} 1964".
pub fn size_words(size: PixelSize) -> String {
    format!("{} \u{d7} {}", size.width, size.height)
}

impl DisplayRotation {
    /// How a rotation reads.
    pub fn words(self) -> &'static str {
        match self {
            DisplayRotation::Standard => "Standard",
            DisplayRotation::Quarter => "90\u{b0}",
            DisplayRotation::Half => "180\u{b0}",
            DisplayRotation::ThreeQuarters => "270\u{b0}",
        }
    }
}

impl ArrangeSide {
    /// How a side reads before the name of the display it is a side of.
    pub fn words(self) -> &'static str {
        match self {
            ArrangeSide::Right => "Right of",
            ArrangeSide::Left => "Left of",
            ArrangeSide::Above => "Above",
            ArrangeSide::Below => "Below",
        }
    }
}
