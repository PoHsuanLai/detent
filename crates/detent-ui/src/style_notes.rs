//! The line under "Use my styles" that counts what the person's styles have to say, and the sheet
//! that lists it. The files are read when the Appearance page opens (the page is mounted afresh
//! for each visit), off the window's thread; a file that cannot be read has no line.

use crate::env::Env;
use detent_files::{ConfigHome, DataHome, published_classes, read_style_sheets};
use detent_model::{STYLE_NOTES_LEAD, STYLE_NOTES_TITLE, StyleFile, StyleNote, StyleNotes};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds_lint::user_stylesheet_with;
use std::path::PathBuf;

/// The notes quire's lint has on every style file under `home`, in plain words, a class an
/// installed app publishes (`apps`) not counting as an internal one. Reads files: call it off the
/// window's thread.
fn read_notes(home: &ConfigHome, apps: &[Vec<String>]) -> StyleNotes {
    let kits = ds::kits();
    let files = read_style_sheets(home)
        .into_iter()
        .map(|sheet| StyleFile {
            owner: sheet.owner,
            notes: user_stylesheet_with(&sheet.css, &kits, apps)
                .into_iter()
                .map(|note| StyleNote {
                    line: note.line,
                    words: note.kind.plain_words().to_owned(),
                })
                .collect(),
        })
        .collect();
    StyleNotes::new(files)
}

async fn notes_of(home: ConfigHome, data: DataHome, system: Vec<PathBuf>) -> StyleNotes {
    tokio::task::spawn_blocking(move || read_notes(&home, &published_classes(&data, &system)))
        .await
        .unwrap_or_default()
}

/// "3 notes" with a button that opens the sheet of them, or nothing when the styles have none.
#[component]
pub(crate) fn StyleNotesLine() -> Element {
    let env = use_context::<Env>();
    let mut notes = use_signal(StyleNotes::default);
    let mut opened = use_signal(|| Shown::Hidden);
    use_future({
        let env = env.clone();
        move || {
            let env = env.clone();
            async move {
                let (home, data, system) = (
                    env.config_home.clone(),
                    env.data_home.clone(),
                    env.system_data_dirs.clone(),
                );
                notes.set(notes_of(home, data, system).await);
            }
        }
    });
    let Some(summary) = notes().summary() else {
        return rsx! {};
    };
    let close = EventHandler::new(move |()| opened.set(Shown::Hidden));
    rsx! {
        div { class: "style-notes", "data-style-notes": "{summary}",
            FieldGroup {
                FieldRow { label: TextLine::from(summary.clone()),
                    Button {
                        label: "Show",
                        common: Common {
                            aria_label: Some(format!("Show {STYLE_NOTES_TITLE}")),
                            ..Common::default()
                        },
                        onclick: move |_| opened.set(Shown::Visible),
                    }
                }
            }
            if opened() == Shown::Visible {
                Sheet {
                    label: STYLE_NOTES_TITLE.to_owned(),
                    width: SheetWidth::Regular,
                    onclose: close,
                    on_return: close,
                    div { class: "advanced-sheet style-notes-sheet",
                        Label { text: STYLE_NOTES_TITLE, role: LabelRole::Primary, style: LabelStyle::Headline }
                        Label { text: STYLE_NOTES_LEAD, role: LabelRole::Secondary }
                        FieldGroup {
                            for (at, row) in notes().rows().into_iter().enumerate() {
                                FieldRow { key: "{at}", label: TextLine::from(row.words),
                                    Label { text: row.place, role: LabelRole::Secondary }
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
}
