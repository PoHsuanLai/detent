//! The switch "Let my other computers use this computer's models": turning it on asks first, then
//! sets sharing up through porter, and only when that worked is the setting written; turning it
//! off takes sharing away and clears the setting. The steps are `detent-model`'s `sharing_step`.

use crate::copy::sharing::{ALLOW, CANCEL, sharing_question};
use crate::env::Env;
use crate::files::{Edit, Files, store};
use detent_model::{SHARING_KEY, Sharing, SharingEffect, SharingInput, Way, sharing_step};
use dioxus::prelude::*;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;

/// The switch's state and what it needs to act.
#[derive(Clone)]
pub(crate) struct SharingSwitch {
    env: Env,
    files: Signal<Files>,
    toasts: ToastHub,
    state: Signal<Sharing>,
    /// The edit the person made, written once sharing is set up (or taken away).
    pending: Signal<Option<Edit>>,
}

/// The switch of the window whose values are `files`.
pub(crate) fn use_sharing(files: Signal<Files>) -> SharingSwitch {
    SharingSwitch {
        env: use_context::<Env>(),
        files,
        toasts: use_toasts(),
        state: use_signal(Sharing::default),
        pending: use_signal(|| None),
    }
}

fn way_of(edit: &Edit) -> Way {
    match edit.value.as_str() {
        Some("on") => Way::On,
        _ => Way::Off,
    }
}

impl SharingSwitch {
    /// Whether `edit` is the person turning this switch.
    pub(crate) fn owns(&self, edit: &Edit) -> bool {
        edit.entry.spec.path.0 == SHARING_KEY
    }

    /// The person turned the switch.
    pub(crate) fn turned(&self, edit: Edit) {
        let way = way_of(&edit);
        let mut pending = self.pending;
        pending.set(Some(edit));
        self.feed(SharingInput::Switched(way, self.env.lender.existing()));
    }

    fn feed(&self, input: SharingInput) {
        let mut state = self.state;
        let (next, effects) = sharing_step(*state.peek(), input);
        state.set(next);
        for effect in effects {
            self.run(effect);
        }
    }

    fn run(&self, effect: SharingEffect) {
        match effect {
            SharingEffect::Enable => {
                let (me, lender) = (self.clone(), self.env.lender.clone());
                spawn(async move {
                    let done = lender.enable().await;
                    me.feed(SharingInput::Finished(done));
                });
            }
            SharingEffect::Disable => {
                let (me, lender) = (self.clone(), self.env.lender.clone());
                spawn(async move {
                    let done = lender.disable().await;
                    me.feed(SharingInput::Finished(done));
                });
            }
            SharingEffect::Record(way) => {
                let edit = self.pending.peek().clone();
                if let Some(edit) = edit.filter(|edit| way_of(edit) == way) {
                    store(&self.env, self.files, self.toasts, &edit);
                }
            }
            SharingEffect::Tell(words) => self.toasts.push(words, None),
        }
    }

    /// The question while one is asked.
    pub(crate) fn sheet(&self) -> Element {
        let Sharing::Asking(existing) = *self.state.read() else {
            return rsx! {};
        };
        let (title, message) = sharing_question(existing);
        let (allow, cancel) = (self.clone(), self.clone());
        rsx! {
            Alert {
                title,
                message: Some(TextLine::from(message)),
                style: AlertStyle::Informational,
                buttons: vec![
                    AlertButton::new(
                        ALLOW,
                        AlertRole::Normal,
                        EventHandler::new(move |()| allow.feed(SharingInput::Allowed)),
                    ),
                    AlertButton::new(
                        CANCEL,
                        AlertRole::Cancel,
                        EventHandler::new(move |()| cancel.feed(SharingInput::Cancelled)),
                    ),
                ],
            }
        }
    }
}
