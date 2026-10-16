//! What the person reads about a value that waits, and when a page has any.

use super::state::{Trouble, is_memory_setting};
use crate::catalogue::PageModel;
use crate::control::Control;
use crate::entry::KeyEntry;
use crate::words::word_label;
use almanac_core::WaitingSetting;

/// The button that confirms a waiting value.
pub const CONFIRM: &str = "Confirm";

/// Whether any key of `page`, basic or advanced, is a memory setting.
pub fn holds_memory_settings(page: &PageModel) -> bool {
    page.sections
        .iter()
        .chain(&page.advanced)
        .flat_map(|section| &section.keys)
        .any(is_memory_setting)
}

/// `wire`, a value of `entry` as memory names it, the way the row reads it: a number with its
/// unit (`30 days`), a word by its label in curly quotes.
fn read_as(entry: &KeyEntry, wire: &str) -> String {
    let value = entry.control.from_text(wire).ok();
    match (&entry.control, value) {
        (Control::Slider(range), Some(value)) => range.display(&value),
        (Control::Toggle(_) | Control::Segmented(_) | Control::Menu(_), Some(_)) => {
            format!("\u{201c}{}\u{201d}", word_label(&entry.spec.labels, wire))
        }
        _ => wire.to_owned(),
    }
}

/// The note under a row whose value `setting` waits for the person: what to do, and what memory
/// keeps meanwhile.
pub fn waiting_note(entry: &KeyEntry, setting: &WaitingSetting) -> String {
    format!(
        "Waits for you to confirm. Until then, memory keeps {}.",
        read_as(entry, &setting.shipped)
    )
}

impl Trouble {
    /// The calm line under the row when a confirmation did not go through.
    pub fn words(self) -> &'static str {
        match self {
            Trouble::NotAllowed => "Memory would not take this confirmation from here.",
            Trouble::Invalid => "Memory could not accept this value, so it keeps its own.",
            Trouble::Failed => "This could not be confirmed. Try again in a moment.",
        }
    }
}
