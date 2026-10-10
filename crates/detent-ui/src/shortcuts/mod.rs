//! The desktop's shortcuts, as the Keyboard page lists them: groups of rows, each with its chord
//! and who set it, over the person's keymap file.

mod field;
mod list_view;
mod pane;
mod row_view;

pub(crate) use list_view::ShortcutsPane;
