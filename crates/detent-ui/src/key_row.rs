//! One key: its label and help beside the control its kind gets.

use crate::controls::{Control, FileOnlyControl};
use crate::files::{Edit, Files, value_of, written_value};
use crate::memory::ConsentNote;
use detent_model::{KeyEntry, is_memory_setting};
use dioxus::prelude::*;
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::FieldRow;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::ExtraClass;

/// Whether a search found this key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// It did: the row is highlighted.
    Hit,
    /// It did not, or there is no search.
    Miss,
}

/// A row's stable `id`: `scope`, then `text` with every run of other characters as one `-`, so
/// two rows with the same label keep different help ids.
pub(crate) fn row_id(scope: &str, text: &str) -> String {
    let slug: String = text
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("{scope}-{}", slug.trim_matches('-'))
}

/// The class a highlighted row carries.
pub(crate) const HIT_CLASS: &str = "key-hit";

fn common_for(found: Match, entry: &KeyEntry) -> Common {
    Common {
        id: Some(row_id("key", &entry.spec.path.0)),
        extra_class: match found {
            Match::Hit => ExtraClass::parse(HIT_CLASS).ok(),
            Match::Miss => None,
        },
        ..Common::default()
    }
}

/// A `FieldRow` for `entry`, its value read from `files` and its edits sent to `onedit`.
#[component]
pub fn KeyRow(
    entry: KeyEntry,
    found: Match,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let value = value_of(&files(), &entry);
    let help = (!entry.spec.help.0.is_empty()).then(|| TextLine::from(entry.spec.help.0.clone()));
    let sent = entry.clone();
    rsx! {
        FieldRow {
            label: TextLine::from(entry.spec.label.0.clone()),
            help,
            help_shown: HelpShown::OnRequest,
            common: common_for(found, &entry),
            if let Some(written) = written_value(&files(), &entry) {
                FileOnlyControl { value: written }
            } else {
                Control {
                    entry: entry.clone(),
                    value,
                    onchange: move |value| onedit.call(Edit { entry: sent.clone(), value }),
                }
            }
        }
        if is_memory_setting(&entry) {
            ConsentNote { entry: entry.clone() }
        }
    }
}

/// `entry`'s control on its own, with no row around it and no label beside it: for a control
/// that is a whole block of the page (the Wallpaper page's pictures).
#[component]
pub fn KeyBlock(entry: KeyEntry, files: Signal<Files>, onedit: EventHandler<Edit>) -> Element {
    let value = value_of(&files(), &entry);
    let sent = entry.clone();
    rsx! {
        if let Some(written) = written_value(&files(), &entry) {
            FileOnlyControl { value: written }
        } else {
            Control {
                entry: entry.clone(),
                value,
                onchange: move |value| onedit.call(Edit { entry: sent.clone(), value }),
            }
        }
    }
}
