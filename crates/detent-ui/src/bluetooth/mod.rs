//! The Bluetooth page: the switch, the devices this computer knows and the ones nearby, and
//! pairing. BlueZ comes through a handle the environment carries.

mod actions;
mod agent;
mod bluez;
mod drive;
mod link;
mod read;
mod session;
mod sheets;
mod system;
mod view;

pub use link::{BluetoothLink, BtSession, PairingReply, PairingRequest};
pub(crate) use system::OnSystem;
pub(crate) use view::BluetoothPane;
