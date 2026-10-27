//! The Bluetooth page: the switch, the devices this computer knows and the ones nearby, and
//! pairing. BlueZ comes through a handle the environment carries.

mod drive;
mod sheets;
mod view;

pub(crate) use view::BluetoothPane;
