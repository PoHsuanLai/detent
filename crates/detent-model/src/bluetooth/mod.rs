//! The Bluetooth page, as Settings meets it: the switch, the devices this computer knows, the
//! ones nearby while the page is open, and pairing. Pure; BlueZ is the window's (`detent-ui`).

mod model;
mod page;
mod step;
mod wire;

pub use model::{
    BatteryLevel, BtAddress, BtDevice, BtKind, BtLink, BtPairing, BtPower, BtStatus, PairingAnswer,
    PairingAsk, PairingCode, PairingQuestion,
};
pub use page::{BLUETOOTH_SLUG, bluetooth_page};
pub use step::{BtAsking, BtEffect, BtInput, BtReady, BtState, bluetooth_step};
pub use wire::{BtDeviceWire, BtHearing, bt_change_matters, bt_device_of, is_under_adapter};

#[cfg(test)]
mod tests;
