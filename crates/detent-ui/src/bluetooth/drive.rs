//! The Bluetooth page's state and what it needs to act: following BlueZ, looking for devices and
//! hearing pairing questions while the page is shown, and asking BlueZ. The steps are
//! `detent-model`'s `bluetooth_step`.

use super::link::{BluetoothLink, BtSession, PairingReply, PairingRequest};
use crate::env::Env;
use detent_model::{BtEffect, BtInput, BtState, PairingAnswer, bluetooth_step};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::sync::{Arc, Mutex};

/// The page's state and its link.
#[derive(Clone)]
pub(super) struct Driver {
    link: Arc<dyn BluetoothLink>,
    toasts: ToastHub,
    /// Where the answer to the question on the sheet goes.
    waiting: Arc<Mutex<Option<PairingReply>>>,
    pub(super) state: Signal<BtState>,
}

/// The page's state, following BlueZ for as long as the page is shown.
pub(super) fn use_bluetooth() -> Driver {
    let env = use_context::<Env>();
    let driver = Driver {
        link: env.bluetooth.clone(),
        toasts: use_toasts(),
        waiting: Arc::default(),
        state: use_signal(BtState::default),
    };
    use_future({
        let driver = driver.clone();
        move || {
            let driver = driver.clone();
            async move { driver.follow().await }
        }
    });
    driver
}

/// The next question a device asks; never, without a session.
async fn next_ask(session: &mut Option<BtSession>) -> Option<PairingRequest> {
    match session {
        Some(session) => session.asks.recv().await,
        None => std::future::pending().await,
    }
}

impl Driver {
    /// Reads BlueZ now and each time it may have changed. While this runs the page is shown, so
    /// devices are looked for and their questions heard; leaving the page ends this and with it
    /// the session.
    async fn follow(&self) {
        match self.link.changes().await {
            Err(missing) => self.feed(BtInput::Heard(Err(missing))),
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
                            Some(request) => self.asked(request),
                            None => session = None,
                        },
                    }
                }
            }
        }
    }

    async fn reread(&self) {
        self.feed(BtInput::Heard(self.link.read().await));
    }

    /// A device asks something: the sheet shows it, and the answer goes back through `reply`.
    fn asked(&self, request: PairingRequest) {
        if let Ok(mut waiting) = self.waiting.lock() {
            *waiting = Some(request.reply);
        }
        self.feed(BtInput::Asked(request.question));
    }

    fn answer(&self, answer: PairingAnswer) {
        let reply = self
            .waiting
            .lock()
            .ok()
            .and_then(|mut waiting| waiting.take());
        if let Some(reply) = reply {
            reply.send(answer);
        }
    }

    /// Gives the page `input` and carries out what it asks for.
    pub(super) fn feed(&self, input: BtInput) {
        let mut state = self.state;
        let (next, effects) = bluetooth_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            self.run(effect);
        }
    }

    fn run(&self, effect: BtEffect) {
        let me = self.clone();
        match effect {
            BtEffect::SetPower(power) => {
                spawn(async move {
                    let done = me.link.set_power(power).await;
                    me.refused_or_reread(done).await;
                });
            }
            BtEffect::Connect(address) => {
                spawn(async move {
                    let done = me.link.connect(address.clone()).await;
                    me.feed(BtInput::Ended(address, done));
                    me.reread().await;
                });
            }
            BtEffect::Pair(address) => {
                spawn(async move {
                    let done = me.link.pair(address.clone()).await;
                    me.feed(BtInput::Ended(address, done));
                    me.reread().await;
                });
            }
            BtEffect::Disconnect(address) => {
                spawn(async move {
                    let done = me.link.disconnect(address).await;
                    me.refused_or_reread(done).await;
                });
            }
            BtEffect::Forget(address) => {
                spawn(async move {
                    let done = me.link.forget(address).await;
                    me.refused_or_reread(done).await;
                });
            }
            BtEffect::Answer(answer) => self.answer(answer),
            BtEffect::Tell(words) => self.toasts.push(words, None),
        }
    }

    /// BlueZ took a request (read what it says now) or refused it (say why).
    async fn refused_or_reread(&self, done: Result<(), String>) {
        match done {
            Ok(()) => self.reread().await,
            Err(words) => self.feed(BtInput::Refused(words)),
        }
    }
}
