//! The list's state, read from the portal when the page is shown and driven by Remove.

use super::io::{list, revoke};
use super::view::Listed;
use crate::copy::{Subject, absence as absence_words};
use crate::env::Env;
use crate::live::BusSource;
use crate::privacy::use_program_book;
use detent_model::{
    ScreenSharingEffect, ScreenSharingInput, ScreenSharingState, screen_sharing_step,
};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use std::collections::VecDeque;

/// The title of the group, and of its row when the portal is not there.
pub(super) const TITLE: &str = "Screen sharing";

/// The apps the portal lets share the screen without asking again, with a Remove on each.
#[component]
pub(crate) fn ScreenSharingPane() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(ScreenSharingState::default);
    let book = use_program_book();

    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move {
                let found = list(&bus).await;
                drive(bus, state, toasts, ScreenSharingInput::Found(found)).await;
            }
        }
    });
    let remove = EventHandler::new({
        let bus = env.bus.clone();
        move |input: ScreenSharingInput| {
            spawn(drive(bus.clone(), state, toasts, input));
        }
    });

    match state() {
        ScreenSharingState::Looking => rsx! {},
        ScreenSharingState::Unavailable(absence) => rsx! {
            div { class: "screen-sharing", "data-screen-sharing": "unavailable",
                FieldGroup { title: Some(TITLE.to_owned()),
                    FieldRow {
                        label: TextLine::from(TITLE),
                        help: Some(TextLine::from(absence_words(Subject::ScreenSharing, absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        ScreenSharingState::Ready(ready) => rsx! {
            Listed { ready, book: book(), now: env.updates.now(), remove }
        },
    }
}

/// Applies `first`, then every input its effects lead to, until nothing is left to do. The calls
/// to the portal wait here, so a caller spawns this and returns.
async fn drive(
    bus: BusSource,
    mut state: Signal<ScreenSharingState>,
    toasts: ToastHub,
    first: ScreenSharingInput,
) {
    let mut queue = VecDeque::from([first]);
    while let Some(input) = queue.pop_front() {
        let (next, effects) = screen_sharing_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            match effect {
                ScreenSharingEffect::Tell(text) => toasts.push(text, None),
                ScreenSharingEffect::Reload => {
                    queue.push_back(ScreenSharingInput::Found(list(&bus).await));
                }
                ScreenSharingEffect::Revoke(app) => {
                    queue.push_back(ScreenSharingInput::Removed(revoke(&bus, &app).await));
                }
            }
        }
    }
}
