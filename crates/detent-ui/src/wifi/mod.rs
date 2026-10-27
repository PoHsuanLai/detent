//! The Wi-Fi page: the switch, the network this computer is on and the ones in range, joining one
//! (also one that hides its name or signs in with an account), and forgetting one. The network service comes through a handle the environment carries.

mod drive;
mod network_sheets;
mod sheets;
mod view;

pub(crate) use view::WifiPane;
