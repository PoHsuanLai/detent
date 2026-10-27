//! The Wi-Fi pane: the switch, the networks in range, and joining, signing in to, and forgetting
//! one. The steps are `detent-model`'s `wifi_step`; this carries out what they ask over the link.

use crate::live::Live;
use crate::notice::Notice;
use crate::pane::{Machine, Pending, pane};
use detent_model::{WifiEffect, WifiInput, WifiState, wifi_step};
use detent_system::wifi::WifiLink;
use std::sync::Arc;

/// The Wi-Fi pane's handle: cheap to clone, one live value behind it.
#[derive(Debug, Clone)]
pub struct Wifi {
    link: Arc<dyn WifiLink>,
    live: Live<WifiState>,
}

pane!(Wifi, WifiState, follow);

impl Wifi {
    pub(crate) fn over(link: Arc<dyn WifiLink>) -> Wifi {
        Wifi {
            link,
            live: Live::default(),
        }
    }

    /// Gives the pane `input` now; the returned work carries out what it asks for.
    pub fn send(&self, input: WifiInput) -> Pending {
        self.dispatch(input)
    }

    /// Reads the service now and each time it may have changed; the service going away or coming
    /// back is a change too. The radio is asked to look again once.
    async fn follow(&self) {
        match self.link.changes().await {
            Err(missing) => self.dispatch(WifiInput::Heard(Err(missing))).await,
            Ok(mut ticks) => {
                self.reread().await;
                self.link.scan().await;
                while ticks.recv().await.is_some() {
                    self.reread().await;
                }
            }
        }
    }

    async fn reread(&self) {
        self.dispatch(WifiInput::Heard(self.link.read().await))
            .await;
    }

    /// The service's answer to a request that moves something: read again, or say why not.
    async fn read_or_refused(&self, done: Result<(), String>) -> Vec<WifiInput> {
        match done {
            Ok(()) => vec![WifiInput::Heard(self.link.read().await)],
            Err(words) => vec![WifiInput::Refused(words)],
        }
    }
}

impl Machine for Wifi {
    type Input = WifiInput;
    type Effect = WifiEffect;

    fn step(state: WifiState, input: WifiInput) -> (WifiState, Vec<WifiEffect>) {
        wifi_step(state, input)
    }

    async fn run(&self, effect: WifiEffect) -> Vec<WifiInput> {
        match effect {
            WifiEffect::SetRadio(radio) => {
                let done = self.link.set_radio(radio).await;
                self.read_or_refused(done).await
            }
            WifiEffect::Join(ssid, passphrase) => {
                let done = self.link.join(ssid.clone(), passphrase).await;
                vec![
                    WifiInput::Joined(ssid, done),
                    WifiInput::Heard(self.link.read().await),
                ]
            }
            WifiEffect::SignIn(ssid, account) => {
                let done = self.link.sign_in(ssid.clone(), account).await;
                vec![
                    WifiInput::Joined(ssid, done),
                    WifiInput::Heard(self.link.read().await),
                ]
            }
            WifiEffect::JoinHidden(network) => {
                let ssid = network.ssid.clone();
                let done = self.link.join_hidden(network).await;
                vec![
                    WifiInput::Joined(ssid, done),
                    WifiInput::Heard(self.link.read().await),
                ]
            }
            WifiEffect::Forget(ssid) => {
                let done = self.link.forget(ssid).await;
                self.read_or_refused(done).await
            }
            WifiEffect::Tell(words) => {
                self.live.notice(Notice::Words(words));
                Vec::new()
            }
            WifiEffect::CouldNotJoin => {
                self.live.notice(Notice::CouldNotJoin);
                Vec::new()
            }
        }
    }
}
