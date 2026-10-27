//! The line under "Use my styles" that counts what the person's styles have to say, and the sheet
//! that lists it. The files are read when the Appearance page opens (the page is mounted afresh
//! for each visit), off the window's thread; a file that cannot be read has no line.

use crate::copy::style_notes::{STYLE_NOTES_LEAD, STYLE_NOTES_TITLE, note_place, notes_summary};
use crate::env::Env;
use detent_client::StyleInputs;
use detent_model::{StyleFile, StyleNote, StyleNotes};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds_lint::user_stylesheet_with;

/// The notes quire's lint has on every style file `read` found, in plain words, a class an
/// installed app publishes not counting as an internal one.
fn read_notes(read: StyleInputs) -> StyleNotes {
    let StyleInputs { sheets, apps } = read;
    let kits = ds::kits();
    let files = sheets
        .into_iter()
        .map(|sheet| StyleFile {
            owner: sheet.owner,
            notes: user_stylesheet_with(&sheet.css, &kits, &apps)
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
                let read = env
                    .client
                    .read_styles(
                        env.config_home.clone(),
                        env.data_home.clone(),
                        env.system_data_dirs.clone(),
                    )
                    .await;
                notes.set(read_notes(read));
            }
        }
    });
    let total = notes().total();
    if total == 0 {
        return rsx! {};
    }
    let summary = notes_summary(total);
    let close = EventHandler::new(move |()| opened.set(Shown::Hidden));
    rsx! {
        div { class: "style-notes", "data-style-notes": "{summary}",
            FieldGroup {
                FieldRow { label: TextLine::from(summary.clone()),
                    Button {
                        label: crate::copy::style_notes::SHOW,
                        common: Common {
                            aria_label: Some(crate::copy::style_notes::show(STYLE_NOTES_TITLE)),
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
                                FieldRow { key: "{at}", label: TextLine::from(row.words.clone()),
                                    Label { text: note_place(&row), role: LabelRole::Secondary }
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
}
