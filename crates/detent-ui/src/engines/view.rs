//! The note, its install line and its button.

use crate::copy;
use crate::env::Env;
use crate::live::use_live;
use detent_client::Client;
use detent_model::{
    EnginesInput, InstallAdvice, Restart, StartStanding, install_advice, setup_note,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::command_line::CommandLine;
use ds::components::controls::copy_state::CopyState;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

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
    let engines = use_live(Client::engines);
    let state = engines.state;
    let mut copy = use_signal(CopyState::default);
    let advice = use_hook({
        let file = env.os_release.clone();
        move || install_advice(&std::fs::read_to_string(file).unwrap_or_default())
    });
    let Some(note) = setup_note(&state(), &advice) else {
        return rsx! {};
    };
    let (press, allow, cancel) = (
        engines.pane.clone(),
        engines.pane.clone(),
        engines.pane.clone(),
    );
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
                            spawn(allow.send(EnginesInput::Allowed));
                        }),
                    ),
                    AlertButton::new(
                        copy::sharing::CANCEL,
                        AlertRole::Cancel,
                        EventHandler::new(move |()| {
                            spawn(cancel.send(EnginesInput::Cancelled));
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
                        spawn(press.send(EnginesInput::StartUsing));
                    },
                }
            }
        }
        {question}
    }
}
