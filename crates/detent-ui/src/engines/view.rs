//! The note, its install line and its button.

use super::io::{look, restart};
use crate::copy;
use crate::env::Env;
use crate::live::{BusSource, Daemon, connect, next_owner, owner_changes};
use detent_model::{
    EnginesEffect, EnginesInput, EnginesState, InstallAdvice, LlamaCpp, Restart, StartStanding,
    engines_step, install_advice, setup_note,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::command_line::CommandLine;
use ds::components::controls::copy_state::CopyState;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;
use ds::stack::toast_hub::ToastHub;
use ds::style::tokens::control_size::ControlSize;

/// Steps the state on `input` and does what that asks; what each ask ends with is fed back in.
fn drive(mut state: Signal<EnginesState>, toasts: ToastHub, bus: BusSource, input: EnginesInput) {
    let (next, effects) = engines_step(*state.peek(), input);
    state.set(next);
    for effect in effects {
        let bus = bus.clone();
        match effect {
            EnginesEffect::RestartFailed => {
                toasts.push(copy::engines::restart_failed().to_owned(), None)
            }
            EnginesEffect::Look => {
                spawn(async move {
                    let found = look(&bus).await;
                    drive(state, toasts, bus, EnginesInput::Listed(found));
                });
            }
            EnginesEffect::Restart => {
                spawn(async move {
                    let ended = restart(&bus).await;
                    drive(state, toasts, bus, EnginesInput::Restarted(ended));
                });
            }
        }
    }
}

/// Looks when the note appears and again each time the assistant's name changes hands, never
/// starting it.
async fn follow(state: Signal<EnginesState>, toasts: ToastHub, bus: BusSource) {
    let Ok(connection) = connect(&bus).await else {
        return drive(state, toasts, bus, EnginesInput::Listed(LlamaCpp::Unknown));
    };
    // Subscribed before the first look, so a change in between is not lost.
    let Ok(mut owners) = owner_changes(&connection, &Daemon::inferd()).await else {
        return;
    };
    loop {
        let found = look(&bus).await;
        drive(state, toasts, bus.clone(), EnginesInput::Listed(found));
        if next_owner(&mut owners).await.is_none() {
            return;
        }
    }
}

/// What the "Start using it" button does while it is not pressable.
fn availability_of(start: StartStanding) -> Availability {
    match start {
        StartStanding::Ready => Availability::Enabled,
        StartStanding::Working => Availability::Busy,
    }
}

/// The note under "Models on this computer" while the assistant has not found llama.cpp; nothing
/// when it has, when it is not running, or when it does not say.
#[component]
pub(crate) fn LocalEnginesNote() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(EnginesState::default);
    let mut copy = use_signal(CopyState::default);
    let advice = use_hook({
        let file = env.os_release.clone();
        move || install_advice(&std::fs::read_to_string(file).unwrap_or_default())
    });
    use_future({
        let bus = env.bus.clone();
        move || follow(state, toasts, bus.clone())
    });
    let Some(note) = setup_note(&state(), &advice) else {
        return rsx! {};
    };
    let (press, allow, cancel) = (env.bus.clone(), env.bus.clone(), env.bus.clone());
    let question = (state().restart == Restart::Asking).then(|| {
        let (title, message) = copy::engines::restart_question();
        rsx! {
            Alert {
                title,
                message: Some(TextLine::from(message)),
                style: AlertStyle::Informational,
                buttons: vec![
                    AlertButton::new(
                        copy::engines::restart(),
                        AlertRole::Normal,
                        EventHandler::new(move |()| {
                            drive(state, toasts, allow.clone(), EnginesInput::Allowed);
                        }),
                    ),
                    AlertButton::new(
                        copy::sharing::CANCEL,
                        AlertRole::Cancel,
                        EventHandler::new(move |()| {
                            drive(state, toasts, cancel.clone(), EnginesInput::Cancelled);
                        }),
                    ),
                ],
            }
        }
    });
    rsx! {
        div { class: "engine-note", "data-engine-note": "llama-cpp",
            Label { text: copy::engines::setup_line(), role: LabelRole::Secondary, style: LabelStyle::Footnote }
            match note.install {
                InstallAdvice::Command(text) => rsx! {
                    CommandLine {
                        text,
                        copy: copy(),
                        on_copy: move |text: String| {
                            let _ = ds_blitz::clipboard::write_text(&text);
                            copy.set(CopyState::Copied);
                        },
                    }
                },
                InstallAdvice::ReleasesPage => rsx! {
                    Label { text: copy::engines::releases_line(), role: LabelRole::Secondary, style: LabelStyle::Footnote }
                },
            }
            div { class: "engine-note-start",
                Button {
                    label: copy::engines::start_using(),
                    size: ControlSize::Small,
                    availability: availability_of(note.start),
                    onclick: move |_| {
                        drive(state, toasts, press.clone(), EnginesInput::StartUsing);
                    },
                }
            }
        }
        {question}
    }
}
