//! The page's state, kept current from the compositor and driven by the person's choices.

use super::book::use_program_book;
use super::program_view::Listed;
use crate::copy::{Subject, absence as absence_words};
use crate::env::Env;
use crate::live::{Followed, follow};
use detent_model::{PrivacyEffect, PrivacyInput, PrivacyState, privacy_step};
use detent_system::bus::{Daemon, reread};
use detent_system::privacy::PrivacyLink;
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyKind;
use std::collections::VecDeque;
use std::sync::Arc;

/// What the person reads when the compositor is gone as a change is sent.
const NOT_AVAILABLE: &str = "Privacy isn\u{2019}t available right now. Try again in a moment.";

/// The rows whose value is read with the rows: the switches and the administrator's read-outs.
fn is_valued(kind: &KeyKind) -> bool {
    matches!(kind, KeyKind::Toggle { .. } | KeyKind::Fixed { .. })
}

/// The programs that have been answered, with a choice for each power.
#[component]
pub(crate) fn PrivacyPane() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(PrivacyState::default);
    let client = use_signal(|| None::<LiveClient>);
    let book = use_program_book();
    use_future({
        let (bus, privacy) = (env.bus.clone(), env.privacy.clone());
        move || {
            let (bus, privacy) = (bus.clone(), privacy.clone());
            let mut client = client;
            async move {
                follow(
                    &bus,
                    &Daemon::compositor(),
                    is_valued,
                    |followed| match followed {
                        Followed::Opened(opened, described) => {
                            client.set(Some(opened));
                            spawn(drive(
                                state,
                                client,
                                privacy.clone(),
                                toasts,
                                PrivacyInput::Found(Ok(described)),
                            ));
                        }
                        Followed::Gone(absence) => {
                            client.set(None);
                            spawn(drive(
                                state,
                                client,
                                privacy.clone(),
                                toasts,
                                PrivacyInput::Found(Err(absence)),
                            ));
                        }
                        Followed::Changed(..) => {
                            let Some(opened) = client.peek().clone() else {
                                return;
                            };
                            let privacy = privacy.clone();
                            spawn(async move {
                                let found = reread(&opened, is_valued).await;
                                drive(state, client, privacy, toasts, PrivacyInput::Found(found))
                                    .await;
                            });
                        }
                    },
                )
                .await;
            }
        }
    });
    let privacy = env.privacy.clone();
    let pick = EventHandler::new(move |input: PrivacyInput| {
        spawn(drive(state, client, privacy.clone(), toasts, input));
    });

    match state() {
        PrivacyState::Looking => rsx! {},
        PrivacyState::Unavailable(absence) => rsx! {
            div { class: "privacy", "data-privacy": "unavailable",
                FieldGroup { title: Some("Privacy & Security".to_owned()),
                    FieldRow {
                        label: TextLine::from("Privacy & Security"),
                        help: Some(TextLine::from(absence_words(Subject::Privacy, absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        PrivacyState::Ready(ready) => rsx! { Listed { ready, book: book(), pick } },
    }
}

/// Applies `first`, then every input its effects lead to, until nothing is left to do. The calls
/// to the compositor wait here, so a caller spawns this and returns.
async fn drive(
    mut state: Signal<PrivacyState>,
    client: Signal<Option<LiveClient>>,
    privacy: Arc<dyn PrivacyLink>,
    toasts: ToastHub,
    first: PrivacyInput,
) {
    let mut queue = VecDeque::from([first]);
    while let Some(input) = queue.pop_front() {
        let (next, effects) = privacy_step(state.peek().clone(), input);
        state.set(next);
        for effect in effects {
            let opened = client.peek().clone();
            match (effect, opened) {
                (PrivacyEffect::Tell(text), _) => toasts.push(text, None),
                (PrivacyEffect::Reload, Some(opened)) => {
                    queue.push_back(PrivacyInput::Found(reread(&opened, is_valued).await));
                }
                (PrivacyEffect::Set { path, to }, Some(opened)) => {
                    queue.push_back(answer(privacy.set(&opened, &path, to).await));
                }
                (PrivacyEffect::Forget(path), Some(opened)) => {
                    queue.push_back(answer(privacy.forget(&opened, &path).await));
                }
                (PrivacyEffect::Set { .. } | PrivacyEffect::Forget(_), None) => {
                    queue.push_back(PrivacyInput::Failed(NOT_AVAILABLE.to_owned()));
                }
                (PrivacyEffect::Reload, None) => {}
            }
        }
    }
}

fn answer(result: Result<(), String>) -> PrivacyInput {
    match result {
        Ok(()) => PrivacyInput::Done,
        Err(why) => PrivacyInput::Failed(why),
    }
}
