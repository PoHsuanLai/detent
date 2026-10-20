//! How the Bluetooth page reaches BlueZ: one handle the environment carries, so a test hands in a
//! service that only answers from memory.

use crate::ticks::Ticks;
use detent_model::{BtAddress, BtPower, BtStatus, Missing, PairingAnswer, PairingQuestion, Work};
use std::fmt::{self, Debug};
use tokio::sync::{mpsc, oneshot};

/// The way to answer one pairing question. Dropping it unanswered is a no.
pub struct PairingReply(oneshot::Sender<PairingAnswer>);

impl PairingReply {
    /// A reply, and the end that hears it.
    pub fn channel() -> (PairingReply, oneshot::Receiver<PairingAnswer>) {
        let (sender, receiver) = oneshot::channel();
        (PairingReply(sender), receiver)
    }

    /// Gives the answer; a device that stopped asking does not hear it.
    pub fn send(self, answer: PairingAnswer) {
        let _ = self.0.send(answer);
    }
}

impl Debug for PairingReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PairingReply")
    }
}

/// A device asking the person something while it pairs.
#[derive(Debug)]
pub struct PairingRequest {
    /// What it asks.
    pub question: PairingQuestion,
    /// Where the answer goes.
    pub reply: PairingReply,
}

/// The page being open: devices are looked for and pairing questions are heard for as long as
/// this is held. Dropping it stops both.
pub struct BtSession {
    /// The questions devices ask.
    pub asks: mpsc::UnboundedReceiver<PairingRequest>,
    _hold: Box<dyn Send>,
}

impl BtSession {
    /// A session that hears `asks`, and stops what it started when `hold` is dropped.
    pub fn new(asks: mpsc::UnboundedReceiver<PairingRequest>, hold: impl Send + 'static) -> Self {
        BtSession {
            asks,
            _hold: Box::new(hold),
        }
    }
}

impl Debug for BtSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BtSession")
    }
}

/// What the page needs of BlueZ.
pub trait BluetoothLink: Debug + Send + Sync {
    /// The switch and the devices now, or why there are none.
    fn read(&self) -> Work<'_, Result<BtStatus, Missing>>;
    /// A tick each time the standing may have changed, also when the service appears or goes;
    /// only the bus being unreachable is an error.
    fn changes(&self) -> Work<'_, Result<Ticks, Missing>>;
    /// Starts looking for devices and listening for pairing questions, for as long as the session
    /// is held.
    fn open(&self) -> Work<'_, Result<BtSession, Missing>>;
    /// Moves the switch; BlueZ's words when it cannot.
    fn set_power(&self, power: BtPower) -> Work<'_, Result<(), String>>;
    /// Connects to a device this computer is paired with.
    fn connect(&self, address: BtAddress) -> Work<'_, Result<(), String>>;
    /// Disconnects from a device.
    fn disconnect(&self, address: BtAddress) -> Work<'_, Result<(), String>>;
    /// Pairs with a device it sees, then connects. Answers when the pairing has ended.
    fn pair(&self, address: BtAddress) -> Work<'_, Result<(), String>>;
    /// Removes a device, so the computer no longer knows it.
    fn forget(&self, address: BtAddress) -> Work<'_, Result<(), String>>;
}
