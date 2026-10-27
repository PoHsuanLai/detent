//! "Where it may run": the built-in assistant's places, as docket's settings hold them. This
//! computer is always on; each of the person's computers has a switch, on once added; each
//! signed-in cloud account has a switch that is off until the person confirms.

use super::models::use_picker;
use super::outside_view::OutsideAssistants;
use super::place_row::{PlaceEntry, PlaceHandlers, PlaceRow, PlaceView, Switch};
use crate::accounts::{Listed, use_listed};
use crate::computers::{Computers, use_computers};
use crate::env::Env;
use crate::icons::tile_on;
use crate::system::Absent;
use detent_client::{load_places, save_places};
use detent_model::external::{
    CloudAccount, CloudAllowed, ComputerName, ModelChoice, OwnComputer, PlaceKind, PlaceName,
    Toggle, kind_label,
};
use detent_model::{
    AccountsPane, ComputerRow, ComputersInput, ComputersState, Confirm, Ground, PlaceRef,
    PlacesEffect, PlacesInput, PlacesState, Ready, Standing, known_of, offers_for, places_step,
};
use dioxus::prelude::*;
use ds::components::fields::field_row::FieldGroup;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use ds::prelude::*;

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

/// What the line under a computer's name says: the models it lends now.
fn models_line(computer: &ComputerRow) -> String {
    if computer.models.is_empty() {
        crate::copy::assistants::NO_MODELS_RIGHT_NOW.to_owned()
    } else {
        computer.models.join(", ")
    }
}

fn own_view(
    own: &OwnComputer,
    computer: &ComputerRow,
    pane: &AccountsPane,
    picker: Option<&Ready>,
) -> PlaceView {
    let place = PlaceRef::Own(own.name.clone());
    PlaceView {
        name: own.name.0.clone(),
        detail: Some(models_line(computer)),
        standing: Some(match computer.standing {
            Standing::Answering => crate::copy::assistants::READY,
            Standing::Silent => crate::copy::assistants::NOT_ANSWERING,
        }),
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
        standing: None,
    }
}

/// The built-in assistant's places, then the assistants that sign themselves in.
#[component]
pub(crate) fn Assistants() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let listed = use_listed();
    let picker = use_picker();
    let (computers, feed) = use_computers();
    let mut state = use_signal({
        let env = env.clone();
        move || PlacesState::of(load_places(&env.config_home).places)
    });
    let pane = match listed() {
        Listed::Accounts(pane) => pane,
        Listed::Looking | Listed::Absent(_) => AccountsPane::default(),
    };

    let listing = listed_computers(computers());

    // What the machine knows (the service's computers, accountd's accounts) is merged into the
    // setting each time either list changes.
    use_effect({
        let env = env.clone();
        move || {
            let pane = match listed() {
                Listed::Accounts(pane) => pane,
                Listed::Looking | Listed::Absent(_) => AccountsPane::default(),
            };
            let computers = listed_computers(computers());
            let read = load_places(&env.config_home).places;
            let (next, _) = places_step(
                state.peek().clone(),
                PlacesInput::Read(read, known_of(&pane, &computers)),
            );
            state.set(next);
        }
    });

    let apply = EventHandler::new({
        let env = env.clone();
        move |input: PlacesInput| {
            let (next, effects) = places_step(state.peek().clone(), input);
            state.set(next);
            for PlacesEffect::Write(places) in effects {
                if let Err(why) = save_places(&env.config_home, &places) {
                    toasts.push(crate::copy::assistants::could_not_save_places(&why), None);
                }
            }
        }
    });

    let current = state();
    let ready = picker();
    let question = current.question();
    let own_rows: Vec<PlaceEntry> = listing
        .iter()
        .filter_map(|computer| {
            let own = current
                .places
                .own
                .iter()
                .find(|own| own.name.0 == computer.name)?;
            Some(PlaceEntry {
                view: own_view(own, computer, &pane, ready.as_ref()),
                on: own_handlers(own.name.clone(), computer.computer.clone(), apply, feed),
            })
        })
        .collect();
    let this_view = PlaceView {
        name: crate::copy::assistants::THIS_COMPUTER.to_owned(),
        detail: None,
        switch: Switch::Always,
        model: current.places.this_computer.clone(),
        offers: ready
            .as_ref()
            .map_or_else(Vec::new, |r| offers_for(&PlaceRef::ThisComputer, &pane, r)),
        ask: None,
        standing: None,
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
                        removed: None,
                    },
                }
            }
            Computers { state: computers(), feed, rows: own_rows }
            if current.places.cloud.is_empty() {
                Absent {
                    face: Some(tile_on(Icon::Globe, Ground::Indigo)),
                    title: kind_label(PlaceKind::Cloud),
                    words: crate::copy::assistants::SIGNED_IN_ACCOUNTS_THAT_OFFER_AI,
                }
            } else {
            FieldGroup { title: Some(kind_label(PlaceKind::Cloud).to_owned()),
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
                            removed: None,
                        },
                    }
                }
            }
            }
            OutsideAssistants { pane: pane.clone() }
            if let Some((title, message)) = question {
                Alert {
                    title,
                    message: Some(TextLine::from(message)),
                    style: AlertStyle::Warning,
                    buttons: vec![
                        AlertButton::new(
                            crate::copy::common::TURN_ON,
                            AlertRole::Normal,
                            EventHandler::new(move |()| apply.call(PlacesInput::Answered(Confirm::Yes))),
                        ),
                        AlertButton::new(
                            crate::copy::common::CANCEL,
                            AlertRole::Cancel,
                            EventHandler::new(move |()| apply.call(PlacesInput::Answered(Confirm::No))),
                        ),
                    ],
                }
            }
        }
    }
}

/// The computers the service lists; none while it is not there.
fn listed_computers(state: ComputersState) -> Vec<ComputerRow> {
    match state {
        ComputersState::Ready(ready) => ready.computers,
        ComputersState::Looking | ComputersState::Unavailable(_) => Vec::new(),
    }
}

fn own_handlers(
    name: PlaceName,
    computer: ComputerName,
    apply: EventHandler<PlacesInput>,
    feed: EventHandler<ComputersInput>,
) -> PlaceHandlers {
    PlaceHandlers {
        switched: {
            let name = name.clone();
            EventHandler::new(move |next: Check| {
                apply.call(PlacesInput::OwnSwitched {
                    name: name.clone(),
                    to: toggle_of(next),
                });
            })
        },
        asked: EventHandler::new(|_| {}),
        picked: EventHandler::new(move |model: Option<ModelChoice>| {
            apply.call(PlacesInput::Model {
                place: PlaceRef::Own(name.clone()),
                model,
            });
        }),
        removed: Some(EventHandler::new(move |()| {
            feed.call(ComputersInput::Remove(computer.clone()));
        })),
    }
}
