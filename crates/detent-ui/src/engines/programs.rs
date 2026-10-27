//! "Programs for local models": the rows that name the program of an engine, which inferd lists
//! as text under "Local models" and which are advanced, so they sit behind a row that opens a
//! sheet. A path is set through the module when the caret leaves the field or Enter is pressed;
//! the module's refusal, in its own words, stays under the field.

use crate::copy;
use crate::key_row::row_id;
use crate::live::{use_client, use_live};
use crate::picker::NoteLine;
use detent_client::{Client, Want};
use detent_model::{Live, ProgramRow, ProgramsInput};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::KeyPath;

/// The row that opens the programs, when inferd lists any; nothing otherwise.
#[component]
pub(crate) fn EnginePrograms() -> Element {
    let lived = use_live(Client::engine_programs);
    let mut opened = use_signal(|| Shown::Hidden);
    let Live::Ready(programs) = (lived.state)() else {
        return rsx! {};
    };
    if programs.rows.is_empty() {
        return rsx! {};
    }
    let close = EventHandler::new(move |()| opened.set(Shown::Hidden));
    let pane = lived.pane.clone();
    let onset = EventHandler::new(move |(path, value): (KeyPath, String)| {
        spawn(pane.send(ProgramsInput::Set(path, value)));
    });
    rsx! {
        div { class: "engine-programs", "data-programs": "list",
            FieldGroup {
                FieldRow {
                    label: TextLine::from(format!("{}\u{2026}", crate::copy::engines::PROGRAMS_FOR_LOCAL_MODELS)),
                    Button {
                        label: crate::copy::common::OPEN,
                        common: Common { aria_label: Some(crate::copy::engines::open(crate::copy::engines::PROGRAMS_FOR_LOCAL_MODELS)), ..Common::default() },
                        onclick: move |_| opened.set(Shown::Visible),
                    }
                }
            }
        }
        if opened() == Shown::Visible {
            Sheet {
                label: crate::copy::engines::PROGRAMS_FOR_LOCAL_MODELS.to_owned(),
                width: SheetWidth::Regular,
                onclose: close,
                div { class: "advanced-sheet", "data-programs": "sheet",
                    Label { text: crate::copy::engines::PROGRAMS_FOR_LOCAL_MODELS, role: LabelRole::Primary, style: LabelStyle::Headline }
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
                            label: crate::copy::common::DONE,
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
    let client = use_client();
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
                label: crate::copy::common::CHOOSE_ELLIPSIS,
                size: ControlSize::Small,
                common: Common {
                    id: Some(row_id("program-choose", &row.path.0)),
                    aria_label: Some(crate::copy::engines::choose_a_file_for(&row.label)),
                    ..Common::default()
                },
                onclick: move |_| {
                    let (chooser, picked) = (client.chooser(), picked.clone());
                    spawn(async move {
                        if let Some(path) = chooser.choose(Want::File).await {
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
