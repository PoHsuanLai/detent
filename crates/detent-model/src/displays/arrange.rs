//! Where displays sit beside one another. The compositor has one space that every display has a
//! place in; the page offers a side of another display rather than coordinates.

use super::model::{DisplayPosition, Monitor};

/// Which side of another display a display sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrangeSide {
    /// To its right.
    Right,
    /// To its left.
    Left,
    /// Above it.
    Above,
    /// Below it.
    Below,
}

impl ArrangeSide {
    /// Every side, in the order the page lists them.
    pub const ALL: [ArrangeSide; 4] = [
        ArrangeSide::Right,
        ArrangeSide::Left,
        ArrangeSide::Above,
        ArrangeSide::Below,
    ];
}

fn as_offset(pixels: u32) -> i32 {
    i32::try_from(pixels).unwrap_or(i32::MAX)
}

/// Where `display` goes to sit on `side` of `other`, edge to edge and top or left aligned. `None`
/// when either does not say how big it is.
pub fn place_beside(
    display: &Monitor,
    other: &Monitor,
    side: ArrangeSide,
) -> Option<DisplayPosition> {
    let (mine, theirs) = (display.logical_size()?, other.logical_size()?);
    let (mine_w, mine_h) = (as_offset(mine.width), as_offset(mine.height));
    let (their_w, their_h) = (as_offset(theirs.width), as_offset(theirs.height));
    let at = other.position;
    Some(match side {
        ArrangeSide::Right => DisplayPosition {
            x: at.x.saturating_add(their_w),
            y: at.y,
        },
        ArrangeSide::Left => DisplayPosition {
            x: at.x.saturating_sub(mine_w),
            y: at.y,
        },
        ArrangeSide::Above => DisplayPosition {
            x: at.x,
            y: at.y.saturating_sub(mine_h),
        },
        ArrangeSide::Below => DisplayPosition {
            x: at.x,
            y: at.y.saturating_add(their_h),
        },
    })
}

/// The side of `other` that `display` sits on, when it sits edge to edge against it.
pub fn side_of(display: &Monitor, other: &Monitor) -> Option<ArrangeSide> {
    ArrangeSide::ALL
        .into_iter()
        .find(|side| place_beside(display, other, *side) == Some(display.position))
}
