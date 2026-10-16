//! The model picker on the Intelligence page: a card per kind of work, a row per tier, and the
//! footer that says why the last answer came from the model it did.

use super::drive::{Picker, use_picker};
use super::tier_row::TierRow;
use crate::env::Env;
use crate::files::{Files, value_of};
use detent_model::{Absence, Adding, Clearing, KindRows, PickerInput, PickerState, Reason};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;

/// The row that turns the footer on.
const SHOW_REASON: &str = "ai.auto.show_reason";

/// Whether the footer is shown: `ai.auto.show_reason`, on when no schema or file says otherwise
/// (inferd's own default).
pub(crate) fn show_reason(env: &Env, files: &Files) -> Shown {
    let on = env
        .catalogue
        .entries()
        .map(|(_, entry)| entry)
        .find(|entry| entry.spec.path.0 == SHOW_REASON)
        .is_none_or(|entry| value_of(files, entry).as_str() == Some("on"));
    if on { Shown::Visible } else { Shown::Hidden }
}

/// inferd's model rows, or why they are not there. `reasons` is whether the footer is shown.
#[component]
pub fn ModelPicker(reasons: Shown) -> Element {
    let env = use_context::<Env>();
    let Picker { state, ask } = use_picker();
    let mut reason = use_signal(|| None::<Reason>);

    let feed = env.reasons.clone();
    use_future(move || {
        let taken = feed.take();
        async move {
            let Some(mut events) = taken else { return };
            while let Some(event) = events.recv().await {
                if let Some(next) = Reason::of(&event) {
                    reason.set(Some(next));
                }
            }
        }
    });

    match state() {
        PickerState::Probing => rsx! {},
        PickerState::Unavailable(absence) => rsx! { Unavailable { absence } },
        PickerState::Ready(ready) => {
            let applying = ready.applying.clone();
            let adding = ready.adding();
            rsx! {
                div { class: "picker", "data-picker": "ready",
                    for (at, kind) in ready.kinds.iter().cloned().enumerate() {
                        KindCard {
                            key: "{kind.area:?}/{kind.label}",
                            titled: at == 0 || ready.kinds[at - 1].area != kind.area,
                            kind,
                            applying: applying.clone(),
                            adding,
                            onask: ask,
                        }
                    }
                    if reasons == Shown::Visible {
                        ReasonFooter { reason: reason() }
                    }
                }
            }
        }
    }
}

/// What the absence of the picker says.
fn absence_help(absence: &Absence) -> String {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop.".to_owned(),
        Absence::NotRunning | Absence::Unavailable => "Models are not available now.".to_owned(),
        Absence::NotPermitted => "Open Settings from the desktop to choose models.".to_owned(),
        Absence::Failed(_) => "Models did not answer. Try again in a moment.".to_owned(),
    }
}

/// The picker's place when the model service cannot be reached: one row that says so, blocking nothing.
#[component]
pub(crate) fn Unavailable(absence: Absence) -> Element {
    rsx! {
        div { class: "picker", "data-picker": "unavailable",
            FieldGroup { title: Some("Model choice".to_owned()),
                FieldRow {
                    label: TextLine::from("Models"),
                    help: Some(TextLine::from(absence_help(&absence))),
                    availability: Availability::Disabled,
                    Label { text: "Unavailable", role: LabelRole::Secondary }
                }
            }
        }
    }
}

/// The title over the slots of one area ("Models").
#[component]
fn AreaTitle(title: String) -> Element {
    rsx! {
        div { class: "picker-area", "data-area": "{title}",
            Label { text: title.clone(), role: LabelRole::Primary }
        }
    }
}

/// One slot: a card of its tiers.
#[component]
fn KindCard(
    kind: KindRows,
    titled: bool,
    applying: Option<detent_model::Applying>,
    adding: Adding,
    onask: EventHandler<PickerInput>,
) -> Element {
    let area = kind.area.clone().filter(|_| titled);
    rsx! {
        if let Some(title) = area {
            AreaTitle { title }
        }
        div { class: "picker-kind",
            FieldGroup { title: Some(kind.label.clone()),
                for row in kind.rows.iter().cloned() {
                    TierRow { key: "{row.path.0}", applying: applying.clone(), adding, clearing: Clearing::Offered, row, onask }
                }
            }
        }
    }
}

/// The line under the picker: why the last model answered; nothing until one has.
#[component]
fn ReasonFooter(reason: Option<Reason>) -> Element {
    let Some(reason) = reason else {
        return rsx! {};
    };
    let text = reason.line();
    rsx! {
        div { class: "picker-reason",
            Label { text, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
        }
    }
}
