//! The Spaces list: one row per Space with Remove, or why the list is not there.

use super::io::{list, remove};
use super::sheet_view::{RemovalSheetView, SheetHandlers};
use crate::env::Env;
use detent_model::{
    Absence, RemovalStage, SpaceKey, SpaceRow, SpacesEffect, SpacesInput, SpacesReady, SpacesState,
    spaces_step,
};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::controls::button_model::ButtonRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// What the absence of the registry says.
fn absence_help(absence: &Absence) -> &'static str {
    match absence {
        Absence::NoBus => {
            "Settings is not connected to the desktop, so there are no Spaces to show."
        }
        Absence::NotRunning => {
            "The accounts service is not running, so there are no Spaces to show."
        }
        Absence::NotPermitted => {
            "Spaces can only be managed when Settings is opened from the desktop."
        }
        Absence::Unavailable => {
            "The accounts service could not be confirmed, so there are no Spaces to show."
        }
        Absence::Failed(_) => "The accounts service did not answer. Try again in a moment.",
    }
}

/// The Spaces the desktop keeps, with a Remove on each.
#[component]
pub(crate) fn SpacesPane() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let mut state = use_signal(SpacesState::default);

    let drive = EventHandler::new({
        let bus = env.bus.clone();
        move |input: SpacesInput| {
            let (next, effects) = spaces_step(state.peek().clone(), input);
            state.set(next);
            for effect in effects {
                let bus = bus.clone();
                match effect {
                    SpacesEffect::Tell(text) => toasts.push(text, None),
                    SpacesEffect::Reload => {
                        spawn(async move {
                            let found = list(&bus).await;
                            let (next, _) =
                                spaces_step(state.peek().clone(), SpacesInput::Found(found));
                            state.set(next);
                        });
                    }
                    SpacesEffect::Remove { space, removal } => {
                        spawn(async move {
                            let ended = remove(&bus, &space, &removal).await;
                            let (next, effects) =
                                spaces_step(state.peek().clone(), SpacesInput::Removed(ended));
                            state.set(next);
                            for effect in effects {
                                match effect {
                                    SpacesEffect::Tell(text) => toasts.push(text, None),
                                    SpacesEffect::Reload => {
                                        let found = list(&bus).await;
                                        let (next, _) = spaces_step(
                                            state.peek().clone(),
                                            SpacesInput::Found(found),
                                        );
                                        state.set(next);
                                    }
                                    SpacesEffect::Remove { .. } => {}
                                }
                            }
                        });
                    }
                }
            }
        }
    });

    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move {
                drive.call(SpacesInput::Found(list(&bus).await));
            }
        }
    });

    match state() {
        SpacesState::Looking => rsx! {},
        SpacesState::Unavailable(absence) => rsx! {
            div { class: "spaces", "data-spaces": "unavailable",
                FieldGroup { title: Some("Spaces".to_owned()),
                    FieldRow {
                        label: TextLine::from("Spaces"),
                        help: Some(TextLine::from(absence_help(&absence))),
                        availability: Availability::Disabled,
                        Label { text: "Unavailable", role: LabelRole::Secondary }
                    }
                }
            }
        },
        SpacesState::Ready(ready) => rsx! { Listed { ready, drive } },
    }
}

#[component]
fn Listed(ready: SpacesReady, drive: EventHandler<SpacesInput>) -> Element {
    let busy = if matches!(ready.stage, RemovalStage::Removing(_)) {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    rsx! {
        div { class: "spaces", "data-spaces": "list",
            if ready.rows.is_empty() {
                EmptyState {
                    title: "No Spaces",
                    description: "A Space you make in an app appears here.",
                }
            } else {
                FieldGroup { title: Some("Spaces".to_owned()),
                    for row in ready.rows.iter().cloned() {
                        SpaceLine { key: "{row.id.0}", row, availability: busy, drive }
                    }
                }
            }
            if let RemovalStage::Asking(sheet) = ready.stage.clone() {
                RemovalSheetView {
                    sheet,
                    on: SheetHandlers {
                        changed: EventHandler::new(move |change| drive.call(SpacesInput::Sheet(change))),
                        confirm: EventHandler::new(move |()| drive.call(SpacesInput::Confirm)),
                        cancel: EventHandler::new(move |()| drive.call(SpacesInput::Cancel)),
                    },
                }
            }
        }
    }
}

#[component]
fn SpaceLine(
    row: SpaceRow,
    availability: Availability,
    drive: EventHandler<SpacesInput>,
) -> Element {
    let key: SpaceKey = row.id.clone();
    rsx! {
        FieldRow { label: TextLine::from(row.name.clone()),
            Button {
                common: Common {
                    aria_label: Some(format!("Remove {}", row.name)),
                    ..Common::default()
                },
                label: "Remove",
                role: ButtonRole::Destructive,
                size: ControlSize::Small,
                availability,
                onclick: move |_| drive.call(SpacesInput::AskRemove(key.clone())),
            }
        }
    }
}
