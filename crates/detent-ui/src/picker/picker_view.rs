//! The model picker on the Intelligence page: a card per kind of work, a row per tier, and the
//! footer that says why the last answer came from the model it did.

use super::drive::{Picker, use_picker};
use super::tier_row::TierRow;
use super::turn_on::TurnOnAlert;
use crate::copy::absence::Subject;
use crate::copy::{self, picker};
use crate::engine_install::EngineSheetView;
use crate::env::Env;
use crate::files::{Files, value_of};
use crate::live::use_client;
use crate::system::Absent;
use detent_model::{Absence, Adding, Clearing, KindRows, PickerInput, PickerState, Reason};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::FieldGroup;
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
    let client = use_client();
    let Picker { state, ask } = use_picker();
    let mut reason = use_signal(|| None::<Reason>);

    use_future(move || {
        let mut reasons = client.reasons();
        async move {
            while let Some(next) = reasons.next().await {
                reason.set(Some(next));
            }
        }
    });

    match state() {
        PickerState::Looking => rsx! {},
        PickerState::Unavailable(absence) => rsx! { Unavailable { absence } },
        PickerState::Ready(ready) => {
            let applying = ready.applying().cloned();
            let adding = ready.adding();
            rsx! {
                div { class: "picker", "data-picker": "ready",
                    for (at, kind) in ready.kinds.iter().cloned().enumerate() {
                        KindCard {
                            key: "{kind.area:?}/{kind.label}",
                            heading: if at == 0 || ready.kinds[at - 1].area != kind.area {
                                AreaHeading::Title
                            } else {
                                AreaHeading::Continued
                            },
                            kind,
                            applying: applying.clone(),
                            adding,
                            onask: ask,
                        }
                    }
                    TurnOnAlert { ready: ready.clone(), onask: ask }
                    if reasons == Shown::Visible {
                        ReasonFooter { reason: reason() }
                    }
                    EngineSheetView {}
                }
            }
        }
    }
}

/// The picker's place when the model service cannot be reached: one row that says so, blocking nothing.
#[component]
pub(crate) fn Unavailable(absence: Absence) -> Element {
    rsx! {
        div { class: "picker", "data-picker": "unavailable",
            Absent {
                title: crate::copy::picker::MODELS,
                words: copy::absence::absence(Subject::Models, absence),
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

/// Whether a slot's card opens with the title of its area, or goes on under the one above.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AreaHeading {
    /// The first slot of an area: the area's title comes first.
    Title,
    /// A later slot of the same area.
    Continued,
}

/// One slot: a card of its tiers.
#[component]
fn KindCard(
    kind: KindRows,
    heading: AreaHeading,
    applying: Option<detent_model::Applying>,
    adding: Adding,
    onask: EventHandler<PickerInput>,
) -> Element {
    let area = kind.area.clone().filter(|_| heading == AreaHeading::Title);
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
    let text = picker::reason(&reason);
    rsx! {
        div { class: "picker-reason",
            Label { text, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
        }
    }
}
