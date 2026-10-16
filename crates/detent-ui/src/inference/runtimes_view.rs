//! "On this computer": the language-model runtimes accountd lists (Ollama, llama.cpp, LM Studio),
//! each a row with its mark and state and a button to its account. A stopped one stays, greyed.
//! Under the group, a note while the assistant has not found llama.cpp.

use crate::accounts::{AccountLink, Listed, OpenButton, use_listed};
use crate::engines::LocalEnginesNote;
use detent_model::{Absence, Runtime};
use dioxus::prelude::*;
use ds::base::vocab::Muting;
use ds::components::content::avatar::{Avatar, AvatarSize, AvatarTone, person_hue};
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};

/// The group's title.
const TITLE: &str = "Models on this computer";

/// What the list says before any runtime is found.
const NONE_FOUND: &str = "Nothing is running here yet. Ollama, llama.cpp and LM Studio appear \
    here when they are.";

/// Where the list stands.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Found {
    /// accountd has not answered yet.
    Looking,
    /// accountd cannot be used, and why.
    Absent(Absence),
    /// The local runtimes accountd lists now, none when none runs.
    Runtimes(Vec<Runtime>),
}

/// What accountd lists as runtimes, kept current.
pub(crate) fn use_runtimes() -> Memo<Found> {
    let listed = use_listed();
    use_memo(move || match listed() {
        Listed::Looking => Found::Looking,
        Listed::Absent(absence) => Found::Absent(absence),
        Listed::Accounts(pane) => Found::Runtimes(Runtime::all_of(&pane)),
    })
}

/// The runtimes accountd lists, or why they are not there.
#[component]
pub fn LocalRuntimes() -> Element {
    let link = use_context::<AccountLink>();
    let found = use_runtimes();

    let group = match found() {
        Found::Looking => rsx! {},
        Found::Absent(absence) => rsx! {
            div { class: "runtimes", "data-runtimes": "unavailable",
                FieldGroup { title: Some(TITLE.to_owned()),
                    FieldRow {
                        label: TextLine::from("Local models"),
                        help: Some(TextLine::from(absence_help(&absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        Found::Runtimes(runtimes) if runtimes.is_empty() => rsx! {
            div { class: "runtimes", "data-runtimes": "empty",
                FieldGroup { title: Some(TITLE.to_owned()),
                    FieldRow {
                        label: TextLine::from("Local models"),
                        help: Some(TextLine::from(NONE_FOUND)),
                        Label { text: "None", role: LabelRole::Tertiary }
                    }
                }
            }
        },
        Found::Runtimes(runtimes) => rsx! {
            div { class: "runtimes", "data-runtimes": "list",
                FieldGroup { title: Some(TITLE.to_owned()),
                    for runtime in runtimes {
                        RuntimeRow {
                            key: "{runtime.key.0}",
                            runtime: runtime.clone(),
                            onopen: move |()| link.open(runtime.key.clone()),
                        }
                    }
                }
            }
        },
    };
    rsx! {
        {group}
        LocalEnginesNote {}
    }
}

/// What the absence of the account service says here.
fn absence_help(absence: &Absence) -> String {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop.".to_owned(),
        Absence::NotRunning => {
            "The accounts service is not running, so nothing on this computer can be listed."
                .to_owned()
        }
        Absence::NotPermitted => "Models on this computer can only be listed when Settings is \
            opened from the desktop. Close this window and open Settings again from the launcher."
            .to_owned(),
        Absence::Unavailable => "The accounts service could not be confirmed, so nothing on this \
            computer can be listed."
            .to_owned(),
        Absence::Failed(_) => {
            "The accounts service did not answer. Try again in a moment.".to_owned()
        }
    }
}

/// One runtime: its round mark, how it stands, and the way to its account. A stopped runtime has
/// a muted mark and a quiet state, and still opens.
#[component]
fn RuntimeRow(runtime: Runtime, onopen: EventHandler<()>) -> Element {
    let running = runtime.is_running();
    let (muting, role, state) = if running {
        (Muting::Audible, LabelRole::Secondary, "running")
    } else {
        (Muting::Muted, LabelRole::Tertiary, "offline")
    };
    let initial = runtime
        .label
        .chars()
        .next()
        .unwrap_or('?')
        .to_ascii_uppercase();
    let common = Common {
        data: DataName::parse("runtime")
            .map(|name| vec![DataAttr::new(name, state)])
            .unwrap_or_default(),
        ..Common::default()
    };
    rsx! {
        FieldRow {
            label: TextLine::from(runtime.label.clone()),
            common,
            Avatar {
                initial,
                size: AvatarSize::Size28,
                tone: AvatarTone::Person(person_hue(&runtime.label)),
                muting,
            }
            Label { text: runtime.state_label(), role, style: LabelStyle::Footnote }
            OpenButton { label: "Manage", subject: runtime.label.clone(), onpress: move |()| onopen.call(()) }
        }
    }
}
