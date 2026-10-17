//! One app that may share the screen without asking, read from the portal's list.

use crate::privacy::ProgramSubject;
use crate::updates::UnixSeconds;

/// The most apps the list holds; the portal remembers no more than this many grants, so a longer
/// list is not the portal's.
pub const GRANTS_MAX: usize = 256;

/// The longest app id taken as one: the portal takes back nothing longer, since an id is a file
/// name.
pub const APP_ID_MAX: usize = 255;

/// How long the portal keeps the answer, as the app asked for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareLength {
    /// While the app is open.
    WhileRunning,
    /// Until the person takes it back.
    UntilRemoved,
}

impl ShareLength {
    /// The length for the portal's persist mode: 1 while the app runs, 2 until taken back. Mode 0
    /// remembers nothing, so it is never listed, and any other number is not the portal's.
    pub fn of_wire(mode: u32) -> Option<ShareLength> {
        match mode {
            1 => Some(ShareLength::WhileRunning),
            2 => Some(ShareLength::UntilRemoved),
            _ => None,
        }
    }
}

/// An app that may share the screen without asking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenGrant {
    /// The app, by the id the portal proved it by. What it is called comes from this and from
    /// system desktop entries (`ProgramBook`), never from what the app or its entry says.
    pub app: ProgramSubject,
    /// How long it was allowed for.
    pub length: ShareLength,
    /// When it was allowed.
    pub granted: UnixSeconds,
}

impl ScreenGrant {
    /// The grant a row of the portal's list stands for; none for a row that is not one: an empty
    /// id, one longer than the portal takes back, a mode it does not make, or a date out of range.
    pub fn of_wire(app: &str, mode: u32, granted: u64) -> Option<ScreenGrant> {
        if app.is_empty() || app.len() > APP_ID_MAX {
            return None;
        }
        Some(ScreenGrant {
            app: ProgramSubject(app.to_owned()),
            length: ShareLength::of_wire(mode)?,
            granted: UnixSeconds(i64::try_from(granted).ok()?),
        })
    }
}
