//! What an advanced section's sheet row is called.

use crate::catalogue::SectionModel;

/// The word for a section that has no title.
const UNTITLED: &str = "More";

/// The name of the sheet row for `section`, and the sheet's title (the row adds the ellipsis).
/// It is the section's title, except where a basic section of the same page has that title too:
/// then the row would read like the basic section, so it says what is inside instead, the one
/// row's own label, or "<Title> Details" for several.
pub fn advanced_name(section: &SectionModel, basic: &[SectionModel]) -> String {
    if section.title.is_empty() {
        return UNTITLED.to_owned();
    }
    let repeats = basic.iter().any(|other| other.title == section.title);
    match (repeats, section.keys.as_slice()) {
        (false, _) => section.title.clone(),
        (true, [only]) => only.spec.label.0.clone(),
        (true, _) => format!("{} Details", section.title),
    }
}
