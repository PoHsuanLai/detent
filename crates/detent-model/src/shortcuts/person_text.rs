//! The person's keymap file with their changes in it, kept as they wrote it everywhere else.

use chordkit::{Action, OverrideEntry, Overrides};

/// The action a line of the file sets, `None` for a comment, a blank line or a line that could
/// not be read: those are kept as they are.
fn action_of(line: &str) -> Option<Action> {
    let (read, problems) = Overrides::parse(line);
    if !problems.is_empty() {
        return None;
    }
    read.entries().first().map(|entry| entry.action.clone())
}

fn lines_of(entry: &OverrideEntry) -> String {
    Overrides::from_entries([entry.clone()]).render()
}

/// `original` with the lines that set an action replaced by `after`'s entry for it, at the place
/// of the action's first line; an action `after` no longer has loses its lines; an action that
/// was not in the file is added at the end. Comments, blank lines and lines that could not be
/// read stay where they are.
pub(super) fn render_person(original: &str, after: &Overrides) -> String {
    let mut written: Vec<&Action> = Vec::new();
    let mut text = String::new();
    for line in original.lines() {
        let Some(action) = action_of(line) else {
            text.push_str(line);
            text.push('\n');
            continue;
        };
        let entry = after.entries().iter().find(|e| e.action == action);
        if let Some(entry) = entry.filter(|e| !written.contains(&&e.action)) {
            text.push_str(&lines_of(entry));
            written.push(&entry.action);
        }
    }
    for entry in after.entries() {
        if !written.contains(&&entry.action) {
            text.push_str(&lines_of(entry));
        }
    }
    text
}
