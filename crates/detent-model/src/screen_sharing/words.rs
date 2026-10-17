//! What the list says under an app's name.

use super::grant::{ScreenGrant, ShareLength};
use crate::updates::{UnixSeconds, age_words};

/// The line under an app: when it was allowed and for how long, such as "Allowed 3 days ago, until
/// you remove it".
pub fn grant_line(grant: &ScreenGrant, now: UnixSeconds) -> String {
    let age = age_words(now, grant.granted);
    match grant.length {
        ShareLength::UntilRemoved => format!("Allowed {age}, until you remove it"),
        ShareLength::WhileRunning => format!("Allowed {age}, while it is open"),
    }
}
