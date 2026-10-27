//! "Programs for local models": the rows that name the program of an engine, which inferd lists
//! as text under "Local models" and which are advanced, so they sit behind a row that opens a
//! sheet. A path is set through the module when the caret leaves the field or Enter is pressed;
//! the module's refusal, in its own words, stays under the field.

use super::io::is_text;
use crate::chooser::{Want, choose};
use crate::copy;
use crate::env::Env;
use crate::inference::{NoteLine, set};
use crate::key_row::row_id;
use crate::live::{BusSource, Daemon, Followed, follow};
use detent_model::{
    Live, ProgramRow, ProgramsEffect, ProgramsInput, ProgramsPane, program_rows, programs_step,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::live::LiveClient;
use ds_settings::schema::KeyPath;

/// What the row that opens the sheet is called, and the sheet.
const TITLE: &str = "Programs for local models";

/// Steps the pane on `input` and does what that asks; what each ask ends with is fed back in.
fn drive(mut pane: Signal<ProgramsPane>, client: Signal<Option<LiveClient>>, input: ProgramsInput) {
    let (next, effects) = programs_step(pane.peek().clone(), input);
    pane.set(next);
    for effect in effects {
        spawn(async move {
            let ProgramsEffect::Set(path, value) = effect;
            let Some(opened) = client.peek().clone() else {
                return;
            };
            let ended = match set(&opened, &path, &value).await {
                Ok(()) => ProgramsInput::Applied(path, value),
                Err(why) => ProgramsInput::Refused(path, why),
            };
            drive(pane, client, ended);
        });
    }
}

/// Follows inferd's module: its text rows with their values, and each value it puts in force.
async fn follow_rows(
    mut pane: Signal<ProgramsPane>,
    mut client: Signal<Option<LiveClient>>,
    bus: BusSource,
) {
    follow(
        &bus,
        &Daemon::inferd(),
        is_text,
        |followed| match followed {
            Followed::Opened(opened, described) => {
                let rows = program_rows(&described);
                let (next, _) = programs_step(pane.peek().clone(), ProgramsInput::Found(Ok(rows)));
                pane.set(next);
                client.set(Some(opened));
            }
            Followed::Gone(absence) => {
                let (next, _) =
                    programs_step(pane.peek().clone(), ProgramsInput::Found(Err(absence)));
                pane.set(next);
                client.set(None);
            }
            Followed::Changed(path, value) => {
                if let toml::Value::String(word) = value {
                    drive(pane, client, ProgramsInput::Changed(path, word));
                }
            }
        },
    )
    .await;
}

/// The row that opens the programs, when inferd lists any; nothing otherwise.
#[component]
pub(crate) fn EnginePrograms() -> Element {
    let env = use_context::<Env>();
    let pane = use_signal(ProgramsPane::default);
    let client = use_signal(|| None::<LiveClient>);
    let mut opened = use_signal(|| Shown::Hidden);
    use_future({
        let bus = env.bus.clone();
        move || follow_rows(pane, client, bus.clone())
    });
    let Live::Ready(programs) = pane() else {
        return rsx! {};
    };
    if programs.rows.is_empty() {
        return rsx! {};
    }
    let close = EventHandler::new(move |()| opened.set(Shown::Hidden));
    let onset = EventHandler::new(move |(path, value): (KeyPath, String)| {
        drive(pane, client, ProgramsInput::Set(path, value));
    });
    rsx! {
        div { class: "engine-programs", "data-programs": "list",
            FieldGroup {
                FieldRow {
                    label: TextLine::from(format!("{TITLE}\u{2026}")),
                    Button {
                        label: "Open",
                        common: Common { aria_label: Some(format!("Open {TITLE}")), ..Common::default() },
                        onclick: move |_| opened.set(Shown::Visible),
                    }
                }
            }
        }
        if opened() == Shown::Visible {
            Sheet {
                label: TITLE.to_owned(),
                width: SheetWidth::Regular,
                onclose: close,
                div { class: "advanced-sheet", "data-programs": "sheet",
                    Label { text: TITLE, role: LabelRole::Primary, style: LabelStyle::Headline }
                    FieldGroup {
                        for row in programs.rows.iter().cloned() {
                            ProgramField {
                                key: "{row.path.0}-{row.value}",
                                refused: programs.refused.get(&row.path).cloned(),
                                row,
                                onset,
                            }
                        }
                    }
                    div { class: "advanced-sheet-buttons",
                        Button {
                            label: "Done",
                            answers: Answers::Return,
                            onclick: move |_| close.call(()),
                        }
                    }
                }
            }
        }
    }
}

/// One program: its field, the button that chooses a file, and the module's refusal under them.
/// The field is made anew when the value in force changes, so it never shows a stale draft.
#[component]
fn ProgramField(
    row: ProgramRow,
    refused: Option<String>,
    onset: EventHandler<(KeyPath, String)>,
) -> Element {
    let env = use_context::<Env>();
    let mut draft = use_signal(|| row.value.clone());
    let (typed, picked) = (row.clone(), row.clone());
    rsx! {
        TextField {
            label: row.label.clone(),
            value: draft(),
            placeholder: copy::engines::found_automatically(),
            help: Some(TextLine::from(row.help.clone())),
            size: ControlSize::Small,
            common: Common { id: Some(row_id("program", &row.path.0)), ..Common::default() },
            oninput: move |next: String| draft.set(next),
            onchange: move |next: String| {
                if next != typed.value {
                    onset.call((typed.path.clone(), next));
                }
            },
        }
        div { class: "system-sheet-buttons",
            Button {
                label: "Choose\u{2026}",
                size: ControlSize::Small,
                common: Common {
                    id: Some(row_id("program-choose", &row.path.0)),
                    aria_label: Some(format!("Choose a file for {}", row.label)),
                    ..Common::default()
                },
                onclick: move |_| {
                    let (bus, picked) = (env.bus.clone(), picked.clone());
                    spawn(async move {
                        if let Some(path) = choose(&bus, Want::File).await {
                            draft.set(path.clone());
                            onset.call((picked.path, path));
                        }
                    });
                },
            }
        }
        if let Some(text) = refused {
            NoteLine { text, kind: "program-refused" }
        }
    }
}
