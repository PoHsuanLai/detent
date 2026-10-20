//! The brightness sill lists for each display (`org.quire.Displays1`): only the displays whose
//! brightness it can read have one, so a display it does not list has no slider.

use super::model::OutputId;

/// A brightness, 0 to 100 percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Brightness(pub u8);

impl Brightness {
    /// `percent` from the bus, held to 100.
    pub fn of_wire(percent: u32) -> Brightness {
        Brightness(u8::try_from(percent.min(100)).unwrap_or(100))
    }
}

/// A slider's travel is a thousand steps.
const TRAVEL: u16 = 1000;

impl Brightness {
    /// Where a slider sits for this brightness, in thousandths of its travel.
    pub fn fraction(self) -> u16 {
        u16::from(self.0.min(100)) * (TRAVEL / 100)
    }

    /// The brightness a slider at `fraction` thousandths of its travel stands for, rounded.
    pub fn of_fraction(fraction: u16) -> Brightness {
        let travel = u32::from(TRAVEL);
        Brightness::of_wire((u32::from(fraction.min(TRAVEL)) * 100 + travel / 2) / travel)
    }
}

/// The brightness of each display sill lists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BrightnessList(Vec<(OutputId, Brightness)>);

impl BrightnessList {
    /// The list as the bus answers it (`a(su)`). A display named twice keeps its first level.
    pub fn of_wire(rows: &[(String, u32)]) -> BrightnessList {
        let mut listed: Vec<(OutputId, Brightness)> = Vec::new();
        for (name, percent) in rows {
            let id = OutputId(name.clone());
            if listed.iter().all(|(seen, _)| seen != &id) {
                listed.push((id, Brightness::of_wire(*percent)));
            }
        }
        BrightnessList(listed)
    }

    /// The brightness of `output`; none when sill does not list it.
    pub fn level_of(&self, output: &OutputId) -> Option<Brightness> {
        self.0
            .iter()
            .find(|(id, _)| id == output)
            .map(|(_, level)| *level)
    }

    /// The list with `output` at `level`, so a slider stays where it was dragged until sill says
    /// otherwise; an output sill does not list stays unlisted.
    pub fn with(mut self, output: &OutputId, level: Brightness) -> BrightnessList {
        if let Some((_, at)) = self.0.iter_mut().find(|(id, _)| id == output) {
            *at = level;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Brightness, BrightnessList};
    use crate::displays::OutputId;

    fn id(name: &str) -> OutputId {
        OutputId(name.to_owned())
    }

    fn wire(rows: &[(&str, u32)]) -> Vec<(String, u32)> {
        rows.iter().map(|(n, p)| ((*n).to_owned(), *p)).collect()
    }

    #[test]
    fn the_answer_becomes_levels() {
        // (what the bus said, [(display, what the slider shows)])
        let cases: Vec<(Vec<(&str, u32)>, Vec<(&str, Option<u8>)>)> = vec![
            (vec![], vec![("eDP-1", None)]),
            (
                vec![("eDP-1", 50), ("HDMI-A-2", 49)],
                vec![("eDP-1", Some(50)), ("HDMI-A-2", Some(49)), ("DP-1", None)],
            ),
            (vec![("eDP-1", 400)], vec![("eDP-1", Some(100))]),
            (vec![("eDP-1", 0)], vec![("eDP-1", Some(0))]),
            (vec![("eDP-1", 30), ("eDP-1", 70)], vec![("eDP-1", Some(30))]),
        ];
        for (rows, shown) in cases {
            let list = BrightnessList::of_wire(&wire(&rows));
            for (name, want) in &shown {
                assert_eq!(
                    list.level_of(&id(name)),
                    (*want).map(Brightness),
                    "{rows:?} {name}"
                );
            }
        }
    }

    #[test]
    fn the_slider_and_the_percentage_agree() {
        // (slider thousandths, percent)
        let cases = [
            (0, 0),
            (4, 0),
            (5, 1),
            (500, 50),
            (996, 100),
            (1000, 100),
            (4000, 100),
        ];
        for (fraction, percent) in cases {
            let want = Brightness(percent);
            assert_eq!(Brightness::of_fraction(fraction), want, "{fraction}");
        }
        for percent in 0..=100 {
            let level = Brightness(percent);
            let back = Brightness::of_fraction(level.fraction());
            assert_eq!(back, level, "{percent}");
        }
    }

    #[test]
    fn a_drag_moves_only_a_listed_display() {
        let list = BrightnessList::of_wire(&wire(&[("eDP-1", 50)]));
        let moved = list.clone().with(&id("eDP-1"), Brightness(20));
        assert_eq!(moved.level_of(&id("eDP-1")), Some(Brightness(20)));
        let unlisted = list.clone().with(&id("DP-1"), Brightness(20));
        assert_eq!(unlisted, list);
    }
}
