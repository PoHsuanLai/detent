//! The Bluetooth page, as Settings meets it: the switch, the devices this computer knows, the
//! ones nearby while the page is open, and pairing. Pure; BlueZ is the window's (`detent-ui`).

mod model;
mod page;
mod step;
mod words;

pub use model::{
    BtAddress, BtDevice, BtKind, BtLink, BtPairing, BtPower, BtStatus, PairingAnswer, PairingAsk,
    PairingCode, PairingQuestion,
};
pub use page::{BLUETOOTH_SLUG, bluetooth_page, is_bluetooth};
pub use step::{BtEffect, BtInput, BtReady, BtSheet, BtState, bluetooth_step};
pub use words::{forget_body, forget_title, missing_words, pairing_body, pairing_title};

#[cfg(test)]
mod tests;
