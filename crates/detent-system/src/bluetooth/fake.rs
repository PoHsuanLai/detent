//! BlueZ from memory: nothing here reaches a bus. It writes down what was asked, moves its own
//! switch, and lets a test make a device ask something.

use super::{BluetoothLink, BtSession, PairingReply, PairingRequest};
use crate::fake_kit::Kit;
use crate::lock::locked;
use crate::ticks::Ticks;
use detent_model::{Absence, BtAddress, BtStatus, PairingAnswer, PairingQuestion, Work};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

/// What was asked of BlueZ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtAsked {
    /// The switch was moved.
    Power(bool),
    /// A device was connected.
    Connect(BtAddress),
    /// A device was disconnected.
    Disconnect(BtAddress),
    /// A device was paired.
    Pair(BtAddress),
    /// A device was forgotten.
    Forget(BtAddress),
}

/// A BlueZ that answers from what it was given.
#[derive(Debug, Clone)]
pub struct FakeBluetooth {
    kit: Arc<Kit<BtStatus, BtAsked>>,
    questions: Arc<Mutex<Option<mpsc::UnboundedSender<PairingRequest>>>>,
}

impl FakeBluetooth {
    /// A BlueZ in this standing.
    pub fn answering(status: BtStatus) -> FakeBluetooth {
        FakeBluetooth::standing(Ok(status))
    }

    /// No BlueZ, for this reason.
    pub fn missing(why: Absence) -> FakeBluetooth {
        FakeBluetooth::standing(Err(why))
    }

    fn standing(standing: Result<BtStatus, Absence>) -> FakeBluetooth {
        FakeBluetooth {
            kit: Kit::new(standing),
            questions: Arc::default(),
        }
    }

    /// The next requests are refused, in these words.
    pub fn refuses(&self, words: &str) {
        self.kit.refuse_with(Some(words.to_owned()));
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<BtAsked> {
        self.kit.asked()
    }

    /// A device asks `question` of the open session; the answer the person gives comes back
    /// through the receiver. `None` when no session is open.
    pub fn ask(&self, question: PairingQuestion) -> Option<oneshot::Receiver<PairingAnswer>> {
        let (reply, answered) = PairingReply::channel();
        let sent = locked(&self.questions)
            .as_ref()
            .map(|sender| sender.send(PairingRequest { question, reply }));
        sent.and_then(Result::ok).map(|()| answered)
    }

    fn take(&self, asked: BtAsked) -> Work<'_, Result<(), String>> {
        self.kit.record(asked);
        Box::pin(std::future::ready(self.kit.admit()))
    }
}

impl BluetoothLink for FakeBluetooth {
    fn read(&self) -> Work<'_, Result<BtStatus, Absence>> {
        Box::pin(std::future::ready(self.kit.standing()))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        Box::pin(std::future::ready(Ok(self.kit.listen())))
    }

    fn open(&self) -> Work<'_, Result<BtSession, Absence>> {
        let (sender, asks) = mpsc::unbounded_channel();
        *locked(&self.questions) = Some(sender);
        Box::pin(std::future::ready(Ok(BtSession::new(asks, ()))))
    }

    fn set_power(&self, on: bool) -> Work<'_, Result<(), String>> {
        let done = self.take(BtAsked::Power(on));
        if self.kit.admit().is_ok() {
            self.kit.edit(|status| status.powered = on);
        }
        done
    }

    fn connect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.take(BtAsked::Connect(address))
    }

    fn disconnect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.take(BtAsked::Disconnect(address))
    }

    fn pair(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.take(BtAsked::Pair(address))
    }

    fn forget(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.take(BtAsked::Forget(address))
    }
}
