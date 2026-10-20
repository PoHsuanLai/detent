//! The Displays page: each display's resolution, refresh rate, rotation and place. The compositor
//! comes through a handle the environment carries.

mod brightness;
mod drive;
mod link;
mod map;
mod system;
mod view;

pub use link::DisplaysLink;
pub(crate) use system::OnCompositor;
pub(crate) use view::DisplaysPane;
