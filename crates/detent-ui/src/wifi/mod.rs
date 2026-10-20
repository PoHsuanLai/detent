//! The Wi-Fi page: the switch, the network this computer is on and the ones in range, and joining
//! one. The network service comes through a handle the environment carries.

mod drive;
mod join;
mod link;
mod nm;
mod read;
mod sheets;
mod system;
mod view;

pub use link::WifiLink;
pub(crate) use system::OnSystem;
pub(crate) use view::WifiPane;
