//! inferd, as the Intelligence page meets it: its live model picker (design/22 section 9.4) and
//! the reasons it gives for a pick.

mod drive;
mod local_rows;
mod note;
mod picker_view;
mod pool_editor;
mod runtimes_view;
mod tier_row;
mod turn_on;

pub(crate) use drive::{Picker, use_picker};
pub(crate) use note::NoteLine;
pub(crate) use picker_view::{ModelPicker, Unavailable, show_reason};
pub(crate) use runtimes_view::LocalRuntimes;
pub(crate) use tier_row::TierRow;
pub(crate) use turn_on::TurnOnAlert;
