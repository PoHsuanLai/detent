//! The Shortcuts list of the Keyboard page: every standard action of the desktop's keymap in
//! groups, each with its chord and whether the standard, this computer or the person set it.
//! The person's changes go to their own keymap file and nowhere else.

mod groups;
mod model;
mod names;
mod person_text;
mod step;

pub use groups::ShortcutGroup;
pub use model::{
    KeymapText, KeymapTexts, MachineLayer, Origin, ReplaceQuestion, ShortcutRow, ShortcutSection,
    ShortcutsState, desktop_keymap, replace_question,
};
pub use names::name_of;
pub use step::{ShortcutsEffect, ShortcutsInput, shortcuts_step};

#[cfg(test)]
mod tests;
