//! "On this computer": the language-model runtimes accountd lists (Ollama, llama.cpp, LM Studio),
//! each a row with its mark and state and a button to its account. A stopped one stays, greyed.

use crate::accounts::{AccountLink, OpenButton, is_valued};
use crate::env::Env;
use crate::live::{Daemon, Followed, follow, reread};
use detent_model::{Absence, AccountsPane, Runtime};
use dioxus::prelude::*;
use ds::base::vocab::Muting;
use ds::components::content::avatar::{Avatar, AvatarSize, AvatarTone, person_hue};
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};
use ds_settings::live::LiveClient;

/// The group's title.
const TITLE: &str = "On this computer";

/// What the list says before any runtime is found.
const NONE_FOUND: &str = "No local runtime is running. Ollama, llama.cpp and LM Studio appear \
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

/// What accountd lists, kept current: read when accountd appears, again when it says a row
/// changed (a runtime came up or stopped), and again when its name changes hands.
pub(crate) fn use_runtimes() -> Signal<Found> {
    let env = use_context::<Env>();
    let mut found = use_signal(|| Found::Looking);
    let mut client = use_signal(|| None::<LiveClient>);
    use_future(move || {
        let bus = env.bus.clone();
        async move {
            follow(
                &bus,
                &Daemon::accountd(),
                is_valued,
                |followed| match followed {
                    Followed::Opened(opened, described) => {
                        client.set(Some(opened));
                        found.set(Found::Runtimes(Runtime::all_of(&AccountsPane::of(
                            &described,
                        ))));
                    }
                    Followed::Gone(absence) => {
                        client.set(None);
                        found.set(Found::Absent(absence));
                    }
                    Followed::Changed(..) => {
                        let Some(opened) = client.peek().clone() else {
                            return;
                        };
                        spawn(async move {
                            if let Ok(described) = reread(&opened, is_valued).await {
                                found.set(Found::Runtimes(Runtime::all_of(&AccountsPane::of(
                                    &described,
                                ))));
                            }
                        });
                    }
                },
            )
            .await;
        }
    });
    found
}

/// The runtimes accountd lists, or why they are not there.
#[component]
pub fn LocalRuntimes() -> Element {
    let link = use_context::<AccountLink>();
    let found = use_runtimes();

    match found() {
        Found::Looking => rsx! {},
        Found::Absent(absence) => rsx! {
            div { class: "runtimes", "data-runtimes": "unavailable",
                FieldGroup { title: Some(TITLE.to_owned()),
                    FieldRow {
                        label: TextLine::from("Local runtimes"),
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
                        label: TextLine::from("Local runtimes"),
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
    }
}

/// What the absence of accountd says here.
fn absence_help(absence: &Absence) -> String {
    match absence {
        Absence::NoBus => "Settings is not connected to the session bus.".to_owned(),
        Absence::NotRunning => {
            "accountd is not running, so no local runtime can be listed.".to_owned()
        }
        Absence::NotPermitted => {
            "accountd does not list Settings among its callers (callers.toml).".to_owned()
        }
        Absence::Failed(why) => format!("accountd could not be asked: {why}"),
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
            OpenButton { label: "Manage", onpress: move |()| onopen.call(()) }
        }
    }
}
