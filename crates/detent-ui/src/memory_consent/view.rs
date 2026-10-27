//! The note under a memory setting whose value waits for the person, with its Confirm button,
//! and the line under the Memory section that says whether memory keeps to its own folders.

use super::drive::MemoryConsent;
use crate::copy::memory::{CONFIRM, confinement_line, trouble_line, waiting_note};
use detent_model::{KeyEntry, kept_value};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// What waits for `entry`'s key, said in plain words with Confirm, and the line that says why
/// the last Confirm did not go through; nothing when nothing waits.
#[component]
pub(crate) fn ConsentNote(entry: KeyEntry) -> Element {
    let consent = use_context::<MemoryConsent>();
    let key = entry.spec.path.0.clone();
    let state = consent.state();
    let Some(setting) = state.waiting_for(&key) else {
        return rsx! {};
    };
    let words = waiting_note(&kept_value(&entry, setting));
    let trouble = state.trouble_for(&key).map(trouble_line);
    let aria = crate::copy::memory::confirm(&entry.spec.label.0);
    let pressed = key.clone();
    rsx! {
        div { class: "consent-note", "data-consent": "{key}",
            div { class: "consent-note-line",
                Label { text: words, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                Button {
                    label: CONFIRM,
                    size: ControlSize::Small,
                    common: Common { aria_label: Some(aria), ..Common::default() },
                    onclick: move |_| consent.confirm(&pressed),
                }
            }
            if let Some(line) = trouble {
                Label { text: line, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
        }
    }
}

/// The quiet line under the Memory section: how far memory keeps itself to its own folders.
/// Nothing when memory is not running, did not answer, or has nothing the person needs to know.
#[component]
pub(crate) fn ConfinementNote() -> Element {
    let consent = use_context::<MemoryConsent>();
    let Some(line) = confinement_line(consent.state().confinement()) else {
        return rsx! {};
    };
    rsx! {
        div { class: "consent-note", "data-confinement": "memory",
            Label { text: line, role: LabelRole::Secondary, style: LabelStyle::Footnote }
        }
    }
}
