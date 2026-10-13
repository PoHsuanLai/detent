//! A page's advanced keys: one row per section, each opening a sheet of that section's rows.

use crate::files::{Edit, Files};
use crate::key_row::KeyRow;
use crate::section_view::{is_hit, match_of};
use detent_model::{PageSlug, SectionModel};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::button_model::Answers;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;
use ds_settings::schema::KeyPath;

/// The word for a section that has no title.
const UNTITLED: &str = "More";

/// The row's label: the section's name and an ellipsis, since the row opens a sheet.
fn row_label(section: &SectionModel) -> String {
    let name = if section.title.is_empty() {
        UNTITLED
    } else {
        section.title.as_str()
    };
    format!("{name}\u{2026}")
}

/// The sheet's title: the section's name.
fn sheet_title(section: &SectionModel) -> String {
    if section.title.is_empty() {
        UNTITLED.to_owned()
    } else {
        section.title.clone()
    }
}

/// The advanced `groups` of the page `slug`, one row each in a group of their own. A search hit
/// inside a section highlights its row.
#[component]
pub fn AdvancedView(
    slug: PageSlug,
    groups: Vec<SectionModel>,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    rsx! {
        div { class: "advanced", "data-page-advanced": "{slug.0}",
            FieldGroup {
                for group in groups.iter().cloned() {
                    SheetRow { key: "{group.title}", section: group, hits: hits.clone(), files, onedit }
                }
            }
        }
    }
}

/// One section's row and, while open, its sheet.
#[component]
fn SheetRow(
    section: SectionModel,
    hits: Vec<KeyPath>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let mut opened = use_signal(|| Shown::Hidden);
    let label = row_label(&section);
    let title = sheet_title(&section);
    let found = section.keys.iter().any(|entry| is_hit(entry, &hits));
    let class = found.then(|| ExtraClass::parse("key-hit").ok()).flatten();
    let close = EventHandler::new(move |()| opened.set(Shown::Hidden));
    rsx! {
        FieldRow {
            label: TextLine::from(label),
            common: Common { extra_class: class, ..Common::default() },
            Button {
                label: "Open",
                common: Common { aria_label: Some(format!("Open {title}")), ..Common::default() },
                onclick: move |_| opened.set(Shown::Visible),
            }
        }
        if opened() == Shown::Visible {
            Sheet {
                label: title.clone(),
                width: SheetWidth::Regular,
                onclose: close,
                on_return: close,
                div { class: "advanced-sheet",
                    Label { text: title.clone(), role: LabelRole::Primary, style: LabelStyle::Headline }
                    FieldGroup {
                        for entry in section.keys.iter().cloned() {
                            KeyRow { key: "{entry.spec.path.0}", found: match_of(&entry, &hits), entry, files, onedit }
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
