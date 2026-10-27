//! The Bluetooth pane: following BlueZ, looking for devices and hearing pairing questions while
//! the pane is followed, and asking BlueZ. The steps are `detent-model`'s `bluetooth_step`.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{BtEffect, BtInput, BtState, PairingAnswer, bluetooth_step};
use detent_system::bluetooth::{BluetoothLink, BtSession, PairingReply, PairingRequest};
use std::sync::{Arc, Mutex, PoisonError};

/// The Bluetooth pane's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Bluetooth {
    link: Arc<dyn BluetoothLink>,
    live: Live<BtState>,
    /// Where the answer to the question on the sheet goes.
    waiting: Arc<Mutex<Option<PairingReply>>>,
}

pane!(Bluetooth, BtState, follow);

/// The next question a device asks; never, without a session.
async fn next_ask(session: &mut Option<BtSession>) -> Option<PairingRequest> {
    match session {
        Some(session) => session.asks.recv().await,
        None => std::future::pending().await,
    }
}

impl Bluetooth {
    pub(crate) fn over(link: Arc<dyn BluetoothLink>) -> Bluetooth {
        Bluetooth {
            link,
            live: Live::default(),
            waiting: Arc::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: BtInput) -> Pending {
        self.dispatch(input)
    }

    /// Reads BlueZ now and each time it may have changed. While this runs the pane is followed, so
    /// devices are looked for and their questions heard; dropping it ends the session.
    async fn follow(&self) {
        match self.link.changes().await {
            Err(missing) => self.dispatch(BtInput::Heard(Err(missing))).await,
            Ok(mut ticks) => {
                self.reread().await;
                let mut session = self.link.open().await.ok();
                loop {
                    tokio::select! {
                        tick = ticks.recv() => {
                            if tick.is_none() {
                                return;
                            }
                            self.reread().await;
                        }
                        asked = next_ask(&mut session) => match asked {
                            Some(request) => self.asked(request).await,
                            None => session = None,
                        },
                    }
                }
            }
        }
    }

    async fn reread(&self) {
        self.dispatch(BtInput::Heard(self.link.read().await)).await;
    }

    /// A device asks something: the sheet shows it, and the answer goes back through `reply`.
    async fn asked(&self, request: PairingRequest) {
        *self.waiting.lock().unwrap_or_else(PoisonError::into_inner) = Some(request.reply);
        self.dispatch(BtInput::Asked(request.question)).await;
    }

    fn answer(&self, answer: PairingAnswer) {
        let reply = self
            .waiting
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(reply) = reply {
            reply.send(answer);
        }
    }

    /// BlueZ took a request (read what it says now) or refused it (say why).
    async fn read_or_refused(&self, done: Result<(), String>) -> Vec<BtInput> {
        match done {
            Ok(()) => vec![BtInput::Heard(self.link.read().await)],
            Err(words) => vec![BtInput::Refused(words)],
        }
    }
}

impl Machine for Bluetooth {
    type Input = BtInput;
    type Effect = BtEffect;

    fn step(state: BtState, input: BtInput) -> (BtState, Vec<BtEffect>) {
        bluetooth_step(state, input)
    }

    async fn run(&self, effect: BtEffect) -> Vec<BtInput> {
        match effect {
            BtEffect::SetPower(power) => {
                let done = self.link.set_power(power).await;
                self.read_or_refused(done).await
            }
            BtEffect::Connect(address) => {
                let done = self.link.connect(address.clone()).await;
                vec![
                    BtInput::Ended(address, done),
                    BtInput::Heard(self.link.read().await),
                ]
            }
            BtEffect::Pair(address) => {
                let done = self.link.pair(address.clone()).await;
                vec![
                    BtInput::Ended(address, done),
                    BtInput::Heard(self.link.read().await),
                ]
            }
            BtEffect::Disconnect(address) => {
                let done = self.link.disconnect(address).await;
                self.read_or_refused(done).await
            }
            BtEffect::Forget(address) => {
                let done = self.link.forget(address).await;
                self.read_or_refused(done).await
            }
            BtEffect::Answer(answer) => {
                self.answer(answer);
                Vec::new()
            }
            BtEffect::Tell(words) => {
                self.live.notice(Notice::Words(words));
                Vec::new()
            }
        }
    }
}
