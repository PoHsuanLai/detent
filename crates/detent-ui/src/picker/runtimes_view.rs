//! "On this computer": the language-model runtimes accountd lists (Ollama, llama.cpp, LM Studio),
//! each a row with its mark and state and a button to its account. A stopped one stays, greyed.
//! Under the group, the models the assistant runs itself with a Start or Stop each, a note while
//! it has not found llama.cpp, and the row that opens the engines' programs.

use crate::accounts::{AccountLink, Listed, OpenButton, use_listed};
use crate::copy;
use crate::copy::absence::Subject;
use crate::engines::{EnginePrograms, LocalEnginesNote};
use crate::running::RunningModels;
use crate::system::Absent;
use detent_model::{Absence, Runtime};
use dioxus::prelude::*;
use ds::base::vocab::Muting;
use ds::components::content::avatar::{Avatar, AvatarSize, AvatarTone, person_hue};
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};

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
                Absent {
                    title: crate::copy::common::LOCAL_MODELS,
                    words: copy::absence::absence(Subject::Runtimes, absence),
                }
            }
        },
        Found::Runtimes(runtimes) if runtimes.is_empty() => rsx! {
            div { class: "runtimes", "data-runtimes": "empty",
                Absent {
                    title: crate::copy::common::LOCAL_MODELS,
                    words: crate::copy::picker::NOTHING_RUNNING_HERE,
                }
            }
        },
        Found::Runtimes(runtimes) => rsx! {
            div { class: "runtimes", "data-runtimes": "list",
                FieldGroup { title: Some(crate::copy::picker::MODELS_ON_THIS_COMPUTER.to_owned()),
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
        RunningModels {}
        LocalEnginesNote {}
        EnginePrograms {}
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
            Label { text: copy::first_run::runtime_state(&runtime), role, style: LabelStyle::Footnote }
            OpenButton { label: crate::copy::common::MANAGE, subject: runtime.label.clone(), onpress: move |()| onopen.call(()) }
        }
    }
}
