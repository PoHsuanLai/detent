//! The page's state, kept current from the compositor and driven by the person's choices.

use super::io::{NOT_AVAILABLE, forget, is_valued, set};
use super::program_view::Listed;
use crate::env::Env;
use crate::live::{Daemon, Followed, follow, reread};
use detent_files::scan_programs;
use detent_model::{Absence, PrivacyEffect, PrivacyInput, PrivacyState, ProgramBook, privacy_step};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use ds_settings::live::LiveClient;
use std::collections::VecDeque;

/// What the absence of the compositor says.
fn absence_help(absence: &Absence) -> &'static str {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop.",
        Absence::NotRunning => "The desktop is not running.",
        Absence::NotPermitted => "Open Settings from the desktop to see Privacy.",
        Absence::Unavailable => "The desktop is not available now.",
        Absence::Failed(_) => "The desktop did not answer. Try again in a moment.",
    }
}

/// The programs that have been answered, with a choice for each power.
#[component]
pub(crate) fn PrivacyPane() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(PrivacyState::default);
    let client = use_signal(|| None::<LiveClient>);
    let mut book = use_signal(ProgramBook::default);

    use_future({
        let dirs = env.application_dirs.clone();
        move || {
            let dirs = dirs.clone();
            async move {
                if let Ok(found) = tokio::task::spawn_blocking(move || scan_programs(&dirs)).await {
                    book.set(found);
                }
            }
        }
    });
    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
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
                                toasts,
                                PrivacyInput::Found(Ok(described)),
                            ));
                        }
                        Followed::Gone(absence) => {
                            client.set(None);
                            spawn(drive(
                                state,
                                client,
                                toasts,
                                PrivacyInput::Found(Err(absence)),
                            ));
                        }
                        Followed::Changed(..) => {
                            let Some(opened) = client.peek().clone() else {
                                return;
                            };
                            spawn(async move {
                                let found = reread(&opened, is_valued).await;
                                drive(state, client, toasts, PrivacyInput::Found(found)).await;
                            });
                        }
                    },
                )
                .await;
            }
        }
    });
    let pick = EventHandler::new(move |input: PrivacyInput| {
        spawn(drive(state, client, toasts, input));
    });

    match state() {
        PrivacyState::Looking => rsx! {},
        PrivacyState::Unavailable(absence) => rsx! {
            div { class: "privacy", "data-privacy": "unavailable",
                FieldGroup { title: Some("Privacy & Security".to_owned()),
                    FieldRow {
                        label: TextLine::from("Privacy & Security"),
                        help: Some(TextLine::from(absence_help(&absence))),
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
                    queue.push_back(answer(set(&opened, &path, to).await));
                }
                (PrivacyEffect::Forget(path), Some(opened)) => {
                    queue.push_back(answer(forget(&opened, &path).await));
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
