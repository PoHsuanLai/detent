//! BlueZ from memory: nothing here reaches a bus. It writes down what was asked, moves its own
//! switch and devices the way BlueZ would, counts the sessions looking for devices, and lets a test
//! make a device ask something.

use super::{BluetoothLink, BtSession, PairingReply, PairingRequest};
use crate::fake_kit::Kit;
use crate::lock::locked;
use crate::ticks::Ticks;
use detent_model::{Absence, BtAddress, BtLink, BtStatus, PairingAnswer, PairingQuestion, Work};
use std::sync::atomic::{AtomicUsize, Ordering};
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
    looking: Arc<AtomicUsize>,
}

/// Counts one session looking for devices until it is dropped.
struct Looking(Arc<AtomicUsize>);

impl Drop for Looking {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
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
            looking: Arc::default(),
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

    /// How many sessions are looking for devices now.
    pub fn looking(&self) -> usize {
        self.looking.load(Ordering::SeqCst)
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

    /// Writes `asked` down and, unless it is refused, moves the standing with `edit`.
    fn take(
        &self,
        asked: BtAsked,
        edit: impl FnOnce(&mut BtStatus),
    ) -> Work<'_, Result<(), String>> {
        self.kit.record(asked);
        let done = self.kit.admit();
        if done.is_ok() {
            self.kit.edit(edit);
        }
        Box::pin(std::future::ready(done))
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
        self.looking.fetch_add(1, Ordering::SeqCst);
        let session = BtSession::new(asks, Looking(self.looking.clone()));
        Box::pin(std::future::ready(Ok(session)))
    }

    fn set_power(&self, on: bool) -> Work<'_, Result<(), String>> {
        self.take(BtAsked::Power(on), |status| status.powered = on)
    }

    fn connect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        let edit = |status: &mut BtStatus| set_link(status, &address, BtLink::Connected);
        self.take(BtAsked::Connect(address.clone()), edit)
    }

    fn disconnect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        let edit = |status: &mut BtStatus| set_link(status, &address, BtLink::Disconnected);
        self.take(BtAsked::Disconnect(address.clone()), edit)
    }

    fn pair(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        let edit = |status: &mut BtStatus| {
            set_link(status, &address, BtLink::Connected);
            for device in status.devices.iter_mut().filter(|d| d.address == address) {
                device.paired = true;
            }
        };
        self.take(BtAsked::Pair(address.clone()), edit)
    }

    fn forget(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        let edit = |status: &mut BtStatus| status.devices.retain(|d| d.address != address);
        self.take(BtAsked::Forget(address.clone()), edit)
    }
}

fn set_link(status: &mut BtStatus, address: &BtAddress, link: BtLink) {
    for device in status.devices.iter_mut().filter(|d| &d.address == address) {
        device.link = link;
    }
}
