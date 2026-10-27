//! The Displays page's words: how a refresh rate, a size, a rotation and a side read.

use detent_model::{ArrangeSide, DisplayRotation, PixelSize, RefreshMilliHz};

/// A refresh rate as a person reads it: "60 Hertz", "59.94 Hertz".
pub(crate) fn hertz(rate: RefreshMilliHz) -> String {
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
pub(crate) fn size(size: PixelSize) -> String {
    format!("{} \u{d7} {}", size.width, size.height)
}

/// How a rotation reads.
pub(crate) fn rotation(rotation: DisplayRotation) -> &'static str {
    match rotation {
        DisplayRotation::Standard => "Standard",
        DisplayRotation::Quarter => "90\u{b0}",
        DisplayRotation::Half => "180\u{b0}",
        DisplayRotation::ThreeQuarters => "270\u{b0}",
    }
}

/// How a side reads before the name of the display it is a side of.
pub(crate) fn side(side: ArrangeSide) -> &'static str {
    match side {
        ArrangeSide::Right => "Right of",
        ArrangeSide::Left => "Left of",
        ArrangeSide::Above => "Above",
        ArrangeSide::Below => "Below",
    }
}

/// The words "Displays".
pub(crate) const DISPLAYS: &str = "Displays";

/// The words "Brightness".
pub(crate) const BRIGHTNESS: &str = "Brightness";

/// The words "Default for display ({size})".
pub(crate) fn default_for_display(size: impl std::fmt::Display) -> String {
    format!("Default for display ({size})")
}

/// The words "Looks like {}".
pub(crate) fn looks_like(a: impl std::fmt::Display) -> String {
    format!("Looks like {a}")
}

/// The words "Resolution".
pub(crate) const RESOLUTION: &str = "Resolution";

/// The words "{words} (Native)".
pub(crate) fn native(words: impl std::fmt::Display) -> String {
    format!("{words} (Native)")
}

/// The words "Display Mode".
pub(crate) const DISPLAY_MODE: &str = "Display Mode";

/// The words "The pixels the display draws".
pub(crate) const THE_PIXELS_THE_DISPLAY_DRAWS: &str = "The pixels the display draws";

/// The words "Refresh Rate".
pub(crate) const REFRESH_RATE: &str = "Refresh Rate";

/// The words "Rotation".
pub(crate) const ROTATION: &str = "Rotation";

/// The words "Position".
pub(crate) const POSITION: &str = "Position";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_rates_read_in_hertz() {
        const CASES: &[(u32, &str)] = &[
            (60_000, "60 Hertz"),
            (59_940, "59.94 Hertz"),
            (143_981, "143.98 Hertz"),
            (74_500, "74.5 Hertz"),
            (59_999, "60 Hertz"),
        ];
        for (milli, want) in CASES {
            assert_eq!(hertz(RefreshMilliHz(*milli)), *want);
        }
    }

    #[test]
    fn a_size_reads_with_the_times_sign() {
        assert_eq!(
            size(PixelSize {
                width: 3024,
                height: 1964
            }),
            "3024 \u{d7} 1964"
        );
    }

    #[test]
    fn rotations_and_sides_read_in_plain_words() {
        let turns = [
            (DisplayRotation::Standard, "Standard"),
            (DisplayRotation::Quarter, "90\u{b0}"),
            (DisplayRotation::Half, "180\u{b0}"),
            (DisplayRotation::ThreeQuarters, "270\u{b0}"),
        ];
        for (turn, want) in turns {
            assert_eq!(rotation(turn), want, "{turn:?}");
        }
        let sides = [
            (ArrangeSide::Right, "Right of"),
            (ArrangeSide::Left, "Left of"),
            (ArrangeSide::Above, "Above"),
            (ArrangeSide::Below, "Below"),
        ];
        for (place, want) in sides {
            assert_eq!(side(place), want, "{place:?}");
        }
    }
}
