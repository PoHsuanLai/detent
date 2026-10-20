//! The pictures of the Wallpaper page, behind one handle that the environment carries, so a test
//! hands in a fake that lists what it is given and never opens the real chooser.

mod desk;
mod system;

pub use desk::{NoPictures, Pictures};
pub(crate) use system::OnDisk;
