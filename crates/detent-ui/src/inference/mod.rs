//! inferd, as the Intelligence page meets it: its live model picker (design/22 section 9.4) and
//! the reasons it gives for a pick.

mod drive;
mod io;
mod local_rows;
mod picker_view;
mod pool_editor;
mod reasons;
mod runtimes_view;
mod tier_row;

pub(crate) use drive::{Picker, use_picker};
pub(crate) use io::{is_choice, set, set_pool};
pub(crate) use picker_view::{AUTOMATIC, ModelPicker, Unavailable, show_reason};
pub use reasons::ReasonFeed;
pub(crate) use runtimes_view::{Found, LocalRuntimes, use_runtimes};
pub(crate) use tier_row::TierRow;
