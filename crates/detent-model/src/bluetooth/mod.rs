//! The Bluetooth page, as Settings meets it: the switch, the devices this computer knows, the
//! ones nearby while the page is open, and pairing. Pure; BlueZ is the window's (`detent-ui`).

mod model;
mod page;
mod step;

pub use model::{
    BatteryLevel, BtAddress, BtDevice, BtKind, BtLink, BtPairing, BtPower, BtStatus, PairingAnswer,
    PairingAsk, PairingCode, PairingQuestion,
};
pub use page::{BLUETOOTH_SLUG, bluetooth_page};
pub use step::{BtAsking, BtEffect, BtInput, BtReady, BtState, bluetooth_step};

#[cfg(test)]
mod tests;
