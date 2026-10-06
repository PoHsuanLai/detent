//! inferd, as the Intelligence page meets it: its live model picker (design/22 section 9.4) and
//! the reasons it gives for a pick.

mod io;
mod picker_view;
mod reasons;
mod runtimes_view;

pub(crate) use io::{is_menu, set};
pub(crate) use picker_view::{ModelPicker, show_reason};
pub use reasons::ReasonFeed;
pub(crate) use runtimes_view::{Found, LocalRuntimes, use_runtimes};
