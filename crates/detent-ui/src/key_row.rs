//! One key: its label and help beside the control its kind gets.

use crate::controls::{Control, FileOnlyControl};
use crate::files::{Edit, Files, value_of, written_value};
use detent_model::KeyEntry;
use dioxus::prelude::*;
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

/// The class a highlighted row carries.
const HIT_CLASS: &str = "key-hit";

fn common_for(found: Match) -> Common {
    Common {
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
            common: common_for(found),
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
}
