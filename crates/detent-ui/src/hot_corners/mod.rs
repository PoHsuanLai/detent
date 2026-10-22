//! The Hot Corners sheet of the Desktop & Dock page: a button at the foot of the page opens a
//! sheet with a picture of the screen and a pop-up button at each of its corners. A corner holds
//! a row for each set of keys; the forms for "Open…", "App Action…" and "Command…" replace the
//! corners inside the same sheet.

mod ask;
mod button;
mod cell;
mod desk;
mod sheet;
mod system;

pub(crate) use button::HotCorners;
pub use desk::{CornerDesk, NoCorners};
pub(crate) use system::OnSystem;
