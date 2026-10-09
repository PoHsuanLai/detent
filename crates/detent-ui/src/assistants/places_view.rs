//! "Where it may run": the built-in assistant's places, as docket's settings hold them. This
//! computer is always on; each of the person's computers has a switch, on once added; each
//! signed-in cloud account has a switch that is off until the person confirms.

use super::external_view::ExternalAssistants;
use super::models::use_picker;
use super::outside_view::OutsideAssistants;
use super::place_row::{PlaceHandlers, PlaceRow, PlaceView, Switch};
use crate::accounts::{Listed, use_listed};
use crate::env::Env;
use detent_files::{load_places, save_places};
use detent_model::{
    AccountsPane, Confirm, PlaceRef, PlacesEffect, PlacesInput, PlacesState, Ready, known_of,
    offers_for, places_step,
};
use dioxus::prelude::*;
use docket_settings::{
    CloudAccount, CloudAllowed, ModelChoice, OwnComputer, PlaceKind, Toggle, kind_label,
};
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;

/// What a group says when it has no place.
const NO_COMPUTERS: &str = "Computers you add appear here. Each starts off.";
const NO_ACCOUNTS: &str = "Signed-in accounts that offer AI appear here. Each starts off.";

fn toggle_of(check: Check) -> Toggle {
    match check {
        Check::On => Toggle::On,
        Check::Off | Check::Mixed => Toggle::Off,
    }
}

fn check_of(toggle: Toggle) -> Check {
    match toggle {
        Toggle::On => Check::On,
        Toggle::Off => Check::Off,
    }
}

fn own_view(own: &OwnComputer, pane: &AccountsPane, picker: Option<&Ready>) -> PlaceView {
    let place = PlaceRef::Own(own.name.clone());
    PlaceView {
        name: own.name.0.clone(),
        detail: None,
        switch: Switch::Is(check_of(own.allowed)),
        model: own.model.clone(),
        offers: picker.map_or_else(Vec::new, |ready| offers_for(&place, pane, ready)),
        ask: None,
    }
}

fn cloud_view(cloud: &CloudAccount, pane: &AccountsPane, picker: Option<&Ready>) -> PlaceView {
    let place = PlaceRef::Cloud(cloud.name.clone());
    let (switch, ask) = match cloud.allowed {
        CloudAllowed::Off => (Check::Off, Check::Off),
        CloudAllowed::On => (Check::On, Check::Off),
        CloudAllowed::AskEachTime => (Check::On, Check::On),
    };
    PlaceView {
        name: cloud.name.0.clone(),
        detail: (!cloud.provider.is_empty()).then(|| cloud.provider.clone()),
        switch: Switch::Is(switch),
        model: cloud.model.clone(),
        offers: picker.map_or_else(Vec::new, |ready| offers_for(&place, pane, ready)),
        ask: Some(ask),
    }
}

/// The built-in assistant's places, then the assistants that sign themselves in.
#[component]
pub(crate) fn Assistants() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let listed = use_listed();
    let picker = use_picker();
    let mut state = use_signal({
        let env = env.clone();
        move || PlacesState::of(load_places(&env.config_home).places)
    });
    let pane = match listed() {
        Listed::Accounts(pane) => pane,
        Listed::Looking | Listed::Absent(_) => AccountsPane::default(),
    };

    // What the machine knows is merged into the setting each time accountd's rows change.
    use_effect({
        let env = env.clone();
        move || {
            if let Listed::Accounts(pane) = listed() {
                let read = load_places(&env.config_home).places;
                let (next, _) = places_step(
                    state.peek().clone(),
                    PlacesInput::Read(read, known_of(&pane)),
                );
                state.set(next);
            }
        }
    });

    let apply = EventHandler::new({
        let env = env.clone();
        move |input: PlacesInput| {
            let (next, effects) = places_step(state.peek().clone(), input);
            state.set(next);
            for PlacesEffect::Write(places) in effects {
                if let Err(why) = save_places(&env.config_home, &places) {
                    toasts.push(
                        format!("Could not save where the assistant may run. {why}"),
                        None,
                    );
                }
            }
        }
    });

    let current = state();
    let ready = picker();
    let question = current.question();
    let this_view = PlaceView {
        name: "This computer".to_owned(),
        detail: None,
        switch: Switch::Always,
        model: current.places.this_computer.clone(),
        offers: ready
            .as_ref()
            .map_or_else(Vec::new, |r| offers_for(&PlaceRef::ThisComputer, &pane, r)),
        ask: None,
    };
    rsx! {
        div { class: "assistants", "data-assistants": "places",
            FieldGroup { title: Some(kind_label(PlaceKind::ThisComputer).to_owned()),
                PlaceRow {
                    view: this_view,
                    on: PlaceHandlers {
                        switched: EventHandler::new(|_| {}),
                        asked: EventHandler::new(|_| {}),
                        picked: EventHandler::new(move |model: Option<ModelChoice>| {
                            apply.call(PlacesInput::Model { place: PlaceRef::ThisComputer, model });
                        }),
                    },
                }
            }
            FieldGroup { title: Some(kind_label(PlaceKind::OwnComputer).to_owned()),
                if current.places.own.is_empty() {
                    EmptyRow { text: NO_COMPUTERS }
                }
                for own in current.places.own.iter().cloned() {
                    PlaceRow {
                        key: "{own.name.0}",
                        view: own_view(&own, &pane, ready.as_ref()),
                        on: PlaceHandlers {
                            switched: {
                                let name = own.name.clone();
                                EventHandler::new(move |next: Check| {
                                    apply.call(PlacesInput::OwnSwitched { name: name.clone(), to: toggle_of(next) });
                                })
                            },
                            asked: EventHandler::new(|_| {}),
                            picked: {
                                let name = own.name.clone();
                                EventHandler::new(move |model: Option<ModelChoice>| {
                                    apply.call(PlacesInput::Model { place: PlaceRef::Own(name.clone()), model });
                                })
                            },
                        },
                    }
                }
            }
            FieldGroup { title: Some(kind_label(PlaceKind::Cloud).to_owned()),
                if current.places.cloud.is_empty() {
                    EmptyRow { text: NO_ACCOUNTS }
                }
                for cloud in current.places.cloud.iter().cloned() {
                    PlaceRow {
                        key: "{cloud.name.0}",
                        view: cloud_view(&cloud, &pane, ready.as_ref()),
                        on: PlaceHandlers {
                            switched: {
                                let name = cloud.name.clone();
                                EventHandler::new(move |next: Check| {
                                    apply.call(PlacesInput::CloudSwitched { name: name.clone(), to: toggle_of(next) });
                                })
                            },
                            asked: {
                                let name = cloud.name.clone();
                                EventHandler::new(move |next: Check| {
                                    apply.call(PlacesInput::AskEachTime { name: name.clone(), to: toggle_of(next) });
                                })
                            },
                            picked: {
                                let name = cloud.name.clone();
                                EventHandler::new(move |model: Option<ModelChoice>| {
                                    apply.call(PlacesInput::Model { place: PlaceRef::Cloud(name.clone()), model });
                                })
                            },
                        },
                    }
                }
            }
            ExternalAssistants { pane: pane.clone() }
            OutsideAssistants {}
            if let Some((title, message)) = question {
                Alert {
                    title,
                    message: Some(TextLine::from(message)),
                    style: AlertStyle::Warning,
                    buttons: vec![
                        AlertButton::new(
                            "Turn On",
                            AlertRole::Normal,
                            EventHandler::new(move |()| apply.call(PlacesInput::Answered(Confirm::Yes))),
                        ),
                        AlertButton::new(
                            "Cancel",
                            AlertRole::Cancel,
                            EventHandler::new(move |()| apply.call(PlacesInput::Answered(Confirm::No))),
                        ),
                    ],
                }
            }
        }
    }
}

/// A line in a group that has no place yet.
#[component]
fn EmptyRow(text: String) -> Element {
    rsx! {
        FieldRow { label: TextLine::from(text), availability: Availability::Disabled,
            Label { text: "None", role: LabelRole::Tertiary, style: LabelStyle::Footnote }
        }
    }
}
