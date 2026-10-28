//! What a row of a system page does while the page is busy.

use ds::prelude::Availability;

/// Whether the page is busy with this row, another row, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowWork {
    /// A press starts something.
    Free,
    /// This row is being worked on (connected, paired or joined).
    Working,
    /// Another row is, so a press waits.
    Held,
}

impl RowWork {
    /// How the row's button looks.
    pub(crate) fn availability(self) -> Availability {
        match self {
            RowWork::Free => Availability::Enabled,
            RowWork::Working => Availability::Busy,
            RowWork::Held => Availability::Disabled,
        }
    }
}
