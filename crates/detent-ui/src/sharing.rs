//! The switch "Let my other computers use this computer's models": turning it on asks first, then
//! sets sharing up through porter, and only when that worked is the setting written; turning it
//! off takes sharing away and clears the setting. The steps are `detent-model`'s `sharing_step`.

use crate::copy::{
    self,
    sharing::{ALLOW, CANCEL, sharing_question},
};
use crate::env::Env;
use crate::files::{Edit, Files, store};
use detent_client::{Heard, Lending, Notice, Pane};
use detent_model::{SHARING_KEY, Sharing, SharingInput, Way};
use dioxus::prelude::*;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;

/// The switch's state and what it needs to act.
#[derive(Clone)]
pub(crate) struct SharingSwitch {
    pane: Lending,
    state: Signal<Sharing>,
    /// The edit the person made, written once sharing is set up (or taken away).
    pending: Signal<Option<Edit>>,
}

/// The switch of the window whose values are `files`.
pub(crate) fn use_sharing(files: Signal<Files>) -> SharingSwitch {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let pane = use_hook(|| env.client.lending());
    let state = use_signal(|| pane.live().state());
    let pending = use_signal(|| None::<Edit>);
    use_future({
        let events = pane.live().events();
        move || {
            let (events, env) = (events.clone(), env.clone());
            let mut state = state;
            async move {
                while let Some(heard) = events.next().await {
                    match heard {
                        Heard::State(next) => state.set(next),
                        Heard::Notice(Notice::Recorded(way)) => {
                            let edit = pending.peek().clone();
                            if let Some(edit) = edit.filter(|edit| way_of(edit) == way) {
                                store(&env, files, toasts, &edit);
                            }
                        }
                        Heard::Notice(other) => {
                            if let Some(words) = copy::notice::notice(&other) {
                                toasts.push(words, None);
                            }
                        }
                    }
                }
            }
        }
    });
    SharingSwitch {
        pane,
        state,
        pending,
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
        spawn(
            self.pane
                .send(SharingInput::Switched(way, self.pane.existing())),
        );
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
                        EventHandler::new(move |()| {
                            spawn(allow.pane.send(SharingInput::Allowed));
                        }),
                    ),
                    AlertButton::new(
                        CANCEL,
                        AlertRole::Cancel,
                        EventHandler::new(move |()| {
                            spawn(cancel.pane.send(SharingInput::Cancelled));
                        }),
                    ),
                ],
            }
        }
    }
}
