//! The Displays page's state and what it needs to act: following the compositor and asking it.
//! The steps are `detent-model`'s `displays_step`.

use super::link::DisplaysLink;
use crate::env::Env;
use detent_model::{DisplaysEffect, DisplaysInput, DisplaysState, displays_step};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::sync::Arc;

/// The page's state and its link.
#[derive(Clone)]
pub(super) struct Driver {
    link: Arc<dyn DisplaysLink>,
    toasts: ToastHub,
    pub(super) state: Signal<DisplaysState>,
}

/// The page's state, following the compositor for as long as the page is shown.
pub(super) fn use_displays() -> Driver {
    let env = use_context::<Env>();
    let driver = Driver {
        link: env.displays.clone(),
        toasts: use_toasts(),
        state: use_signal(DisplaysState::default),
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
    /// Reads the compositor now and each time the displays may have changed.
    async fn follow(&self) {
        match self.link.changes().await {
            Err(missing) => self.feed(DisplaysInput::Heard(Err(missing))),
            Ok(mut ticks) => {
                self.reread().await;
                while ticks.recv().await.is_some() {
                    self.reread().await;
                }
            }
        }
    }

    async fn reread(&self) {
        self.feed(DisplaysInput::Heard(self.link.read().await));
    }

    /// Gives the page `input` and carries out what it asks for.
    pub(super) fn feed(&self, input: DisplaysInput) {
        let mut state = self.state;
        let (next, effects) = displays_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            self.run(effect);
        }
    }

    fn run(&self, effect: DisplaysEffect) {
        let me = self.clone();
        match effect {
            DisplaysEffect::Apply(change) => {
                spawn(async move {
                    let output = change.output.clone();
                    let done = me.link.apply(change).await;
                    me.feed(DisplaysInput::Applied(output, done));
                    me.reread().await;
                });
            }
            DisplaysEffect::Tell(words) => self.toasts.push(words, None),
        }
    }
}
