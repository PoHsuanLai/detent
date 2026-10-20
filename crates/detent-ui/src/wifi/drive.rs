//! The Wi-Fi page's state and what it needs to act: following the service and asking it. The
//! steps are `detent-model`'s `wifi_step`.

use super::link::WifiLink;
use crate::env::Env;
use detent_model::{WifiEffect, WifiInput, WifiState, wifi_step};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::sync::Arc;

/// The page's state and its link.
#[derive(Clone)]
pub(super) struct Driver {
    link: Arc<dyn WifiLink>,
    toasts: ToastHub,
    pub(super) state: Signal<WifiState>,
}

/// The page's state, following the service for as long as the page is shown.
pub(super) fn use_wifi() -> Driver {
    let env = use_context::<Env>();
    let driver = Driver {
        link: env.wifi.clone(),
        toasts: use_toasts(),
        state: use_signal(WifiState::default),
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

impl Driver {
    /// Reads the service now and each time it may have changed; the service going away or coming
    /// back is a change too. The radio is asked to look again once.
    async fn follow(&self) {
        match self.link.changes().await {
            Err(missing) => self.feed(WifiInput::Heard(Err(missing))),
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
        self.feed(WifiInput::Heard(self.link.read().await));
    }

    /// Gives the page `input` and carries out what it asks for.
    pub(super) fn feed(&self, input: WifiInput) {
        let mut state = self.state;
        let (next, effects) = wifi_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            self.run(effect);
        }
    }

    fn run(&self, effect: WifiEffect) {
        let me = self.clone();
        match effect {
            WifiEffect::SetRadio(radio) => {
                spawn(async move {
                    match me.link.set_radio(radio).await {
                        Ok(()) => me.reread().await,
                        Err(words) => me.feed(WifiInput::Refused(words)),
                    }
                });
            }
            WifiEffect::Join(ssid, passphrase) => {
                spawn(async move {
                    let done = me.link.join(ssid.clone(), passphrase).await;
                    me.feed(WifiInput::Joined(ssid, done));
                    me.reread().await;
                });
            }
            WifiEffect::SignIn(ssid, account) => {
                spawn(async move {
                    let done = me.link.sign_in(ssid.clone(), account).await;
                    me.feed(WifiInput::Joined(ssid, done));
                    me.reread().await;
                });
            }
            WifiEffect::JoinHidden(network) => {
                spawn(async move {
                    let ssid = network.ssid.clone();
                    let done = me.link.join_hidden(network).await;
                    me.feed(WifiInput::Joined(ssid, done));
                    me.reread().await;
                });
            }
            WifiEffect::Forget(ssid) => {
                spawn(async move {
                    match me.link.forget(ssid).await {
                        Ok(()) => me.reread().await,
                        Err(words) => me.feed(WifiInput::Refused(words)),
                    }
                });
            }
            WifiEffect::Tell(words) => self.toasts.push(words, None),
        }
    }
}
