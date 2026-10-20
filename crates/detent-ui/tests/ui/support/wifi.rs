//! The network service from memory: nothing here reaches a bus. It writes down what the page asked
//! and moves its own standing the way the real service would.

use detent_model::{
    JoinFailure, Missing, NetworkLink, Passphrase, Ssid, WifiRadio, WifiStatus, Work,
};
use detent_ui::{Ticker, Ticks, WifiLink};
use std::sync::{Arc, Mutex};

/// What the page asked of the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    Radio(WifiRadio),
    Join(Ssid, Option<Passphrase>),
}

#[derive(Debug)]
struct Shared {
    standing: Mutex<Result<WifiStatus, Missing>>,
    asked: Mutex<Vec<Asked>>,
    join_fails: Mutex<Option<JoinFailure>>,
    listeners: Mutex<Vec<Ticker>>,
}

/// A service that answers from what it was given.
#[derive(Debug, Clone)]
pub struct Wifi(Arc<Shared>);

impl Wifi {
    /// A service in this standing.
    pub fn answering(status: WifiStatus) -> Wifi {
        Wifi::standing(Ok(status))
    }

    /// No service, for this reason.
    pub fn missing(why: Missing) -> Wifi {
        Wifi::standing(Err(why))
    }

    fn standing(standing: Result<WifiStatus, Missing>) -> Wifi {
        Wifi(Arc::new(Shared {
            standing: Mutex::new(standing),
            asked: Mutex::default(),
            join_fails: Mutex::default(),
            listeners: Mutex::default(),
        }))
    }

    /// The next join fails like this.
    pub fn join_fails(&self, failure: JoinFailure) {
        *self.0.join_fails.lock().expect("lock") = Some(failure);
    }

    /// What was asked of it, in order.
    pub fn asked(&self) -> Vec<Asked> {
        self.0.asked.lock().expect("lock").clone()
    }

    fn tick(&self) {
        self.0
            .listeners
            .lock()
            .expect("lock")
            .retain(|listener| listener.tick().is_ok());
    }

    fn change(&self, edit: impl FnOnce(&mut WifiStatus)) {
        if let Ok(status) = &mut *self.0.standing.lock().expect("lock") {
            edit(status);
        }
        self.tick();
    }
}

impl WifiLink for Wifi {
    fn read(&self) -> Work<'_, Result<WifiStatus, Missing>> {
        let standing = self.0.standing.lock().expect("lock").clone();
        Box::pin(std::future::ready(standing))
    }

    fn changes(&self) -> Work<'_, Result<Ticks, Missing>> {
        let (listener, ticks) = Ticks::channel();
        self.0.listeners.lock().expect("lock").push(listener);
        Box::pin(std::future::ready(Ok(ticks)))
    }

    fn scan(&self) -> Work<'_, ()> {
        Box::pin(std::future::ready(()))
    }

    fn set_radio(&self, radio: WifiRadio) -> Work<'_, Result<(), String>> {
        self.0.asked.lock().expect("lock").push(Asked::Radio(radio));
        self.change(|status| status.radio = radio);
        Box::pin(std::future::ready(Ok(())))
    }

    fn join(
        &self,
        ssid: Ssid,
        passphrase: Option<Passphrase>,
    ) -> Work<'_, Result<(), JoinFailure>> {
        self.0
            .asked
            .lock()
            .expect("lock")
            .push(Asked::Join(ssid.clone(), passphrase));
        let failure = self.0.join_fails.lock().expect("lock").take();
        if failure.is_none() {
            self.change(|status| {
                for network in &mut status.networks {
                    network.link = if network.ssid == ssid {
                        NetworkLink::Joined
                    } else {
                        NetworkLink::Idle
                    };
                }
            });
        }
        Box::pin(std::future::ready(failure.map_or(Ok(()), Err)))
    }
}
