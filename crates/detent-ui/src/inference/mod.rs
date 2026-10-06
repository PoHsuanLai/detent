//! inferd, as the Intelligence page meets it: the bus to find it on, its live model picker
//! (design/22 section 9.4) and the reasons it gives for a pick.

mod bus;
mod io;
mod picker_view;
mod reasons;

pub use bus::BusSource;
pub(crate) use picker_view::{ModelPicker, show_reason};
pub use reasons::ReasonFeed;
