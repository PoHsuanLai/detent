//! The Wi-Fi page: the switch, the network this computer is on and the ones in range, joining one
//! (also one that hides its name or signs in with an account), and forgetting one. The network service comes through a handle the environment carries.

mod drive;
mod join;
mod known;
mod link;
mod network_sheets;
mod profile;
mod read;
mod sheets;
mod system;
mod view;

pub use link::WifiLink;
pub(crate) use system::OnSystem;
pub(crate) use view::WifiPane;
