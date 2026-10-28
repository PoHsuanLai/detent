//! BlueZ from memory: nothing here reaches a bus. It writes down what the page asked, counts how
//! many sessions look for devices, and lets a test make a device ask something.

use detent_client::{BluetoothLink, BtSession, PairingReply, PairingRequest, Ticker, Ticks};
use detent_model::{Absence, BtAddress, BtLink, BtStatus, PairingAnswer, PairingQuestion, Work};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};

/// What the page asked of BlueZ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    Power(bool),
    Connect(BtAddress),
    Disconnect(BtAddress),
    Pair(BtAddress),
    Forget(BtAddress),
}

#[derive(Debug)]
struct Shared {
    standing: Mutex<Result<BtStatus, Absence>>,
    asked: Mutex<Vec<Asked>>,
    listeners: Mutex<Vec<Ticker>>,
    looking: Arc<AtomicUsize>,
    questions: Mutex<Option<mpsc::UnboundedSender<PairingRequest>>>,
}

/// A BlueZ that answers from what it was given.
#[derive(Debug, Clone)]
pub struct Bluetooth(Arc<Shared>);

/// Counts one session looking for devices until it is dropped.
struct Looking(Arc<AtomicUsize>);

impl Drop for Looking {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

impl Bluetooth {
    /// A BlueZ in this standing.
    pub fn answering(status: BtStatus) -> Bluetooth {
        Bluetooth::standing(Ok(status))
    }

    /// No BlueZ, for this reason.
    pub fn missing(why: Absence) -> Bluetooth {
        Bluetooth::standing(Err(why))
    }

    fn standing(standing: Result<BtStatus, Absence>) -> Bluetooth {
        Bluetooth(Arc::new(Shared {
            standing: Mutex::new(standing),
            asked: Mutex::default(),
            listeners: Mutex::default(),
            looking: Arc::default(),
            questions: Mutex::default(),
        }))
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<Asked> {
        self.0.asked.lock().expect("lock").clone()
    }

    /// How many sessions are looking for devices now.
    pub fn looking(&self) -> usize {
        self.0.looking.load(Ordering::SeqCst)
    }

    /// A device asks `question`; the answer the person gives comes back through the receiver.
    pub fn ask(&self, question: PairingQuestion) -> oneshot::Receiver<PairingAnswer> {
        let (reply, answered) = PairingReply::channel();
        let questions = self.0.questions.lock().expect("lock");
        let sender = questions.as_ref().expect("a session is open");
        sender
            .send(PairingRequest { question, reply })
            .expect("the page listens");
        answered
    }

    fn tick(&self) {
        self.0
            .listeners
            .lock()
            .expect("lock")
            .retain(|listener| listener.tick().is_ok());
    }

    fn change(&self, edit: impl FnOnce(&mut BtStatus)) {
        if let Ok(status) = &mut *self.0.standing.lock().expect("lock") {
            edit(status);
        }
        self.tick();
    }

    fn record(&self, asked: Asked) {
        self.0.asked.lock().expect("lock").push(asked);
    }
}

impl BluetoothLink for Bluetooth {
    fn read(&self) -> Work<'_, Result<BtStatus, Absence>> {
        let standing = self.0.standing.lock().expect("lock").clone();
        Box::pin(std::future::ready(standing))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Absence>> {
        let (listener, ticks) = Ticks::channel();
        self.0.listeners.lock().expect("lock").push(listener);
        Box::pin(std::future::ready(Ok(ticks)))
    }

    fn open(&self) -> Work<'_, Result<BtSession, Absence>> {
        let (sender, asks) = mpsc::unbounded_channel();
        *self.0.questions.lock().expect("lock") = Some(sender);
        self.0.looking.fetch_add(1, Ordering::SeqCst);
        let session = BtSession::new(asks, Looking(self.0.looking.clone()));
        Box::pin(std::future::ready(Ok(session)))
    }

    fn set_power(&self, on: bool) -> Work<'_, Result<(), String>> {
        self.record(Asked::Power(on));
        self.change(|status| status.powered = on);
        Box::pin(std::future::ready(Ok(())))
    }

    fn connect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.record(Asked::Connect(address.clone()));
        self.change(|status| set_link(status, &address, BtLink::Connected));
        Box::pin(std::future::ready(Ok(())))
    }

    fn disconnect(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.record(Asked::Disconnect(address.clone()));
        self.change(|status| set_link(status, &address, BtLink::Disconnected));
        Box::pin(std::future::ready(Ok(())))
    }

    fn pair(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.record(Asked::Pair(address.clone()));
        self.change(|status| {
            set_link(status, &address, BtLink::Connected);
            for device in &mut status.devices {
                if device.address == address {
                    device.paired = true;
                }
            }
        });
        Box::pin(std::future::ready(Ok(())))
    }

    fn forget(&self, address: BtAddress) -> Work<'_, Result<(), String>> {
        self.record(Asked::Forget(address.clone()));
        self.change(|status| status.devices.retain(|device| device.address != address));
        Box::pin(std::future::ready(Ok(())))
    }
}

fn set_link(status: &mut BtStatus, address: &BtAddress, link: BtLink) {
    for device in &mut status.devices {
        if &device.address == address {
            device.link = link;
        }
    }
}
