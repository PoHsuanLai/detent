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
