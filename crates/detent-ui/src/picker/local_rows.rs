//! What Local asks under its row: how models run on this computer, then which model of that
//! way. With nothing set up it says so and offers to add an account that would run them.

use super::note::NoteLine;
use super::tier_row::{NOTHING_CHOSEN, pick_items};
use crate::engine_install::{Board, GET};
use crate::key_row::row_id;
use detent_model::{ChoiceGap, EngineOfferInput, LocalView, PickerInput};
use dioxus::prelude::*;
use ds::components::controls::button::Button;
use ds::components::fields::field_row::FieldRow;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::KeyPath;

/// What the first question is called.
const ENGINE_LABEL: &str = "Runs with";

/// What the second question is called.
const MODEL_LABEL: &str = "Model";

/// What the row is called when nothing on this computer can run a model.
const NO_ENGINE_LABEL: &str = "Local";

/// The button that opens the way to add what would run models here.
const ADD: &str = "Add Account\u{2026}";

/// The questions under the row at `path`: the engine, then, once there is one, its model.
#[component]
pub(super) fn LocalRows(
    path: KeyPath,
    view: LocalView,
    availability: Availability,
    onask: EventHandler<PickerInput>,
) -> Element {
    // The engines the person could get, listed after the ones that run models now: choosing one
    // asks to download it instead of setting anything.
    let board = try_use_context::<Board>();
    let wanted: Vec<(String, String)> = board
        .map(|board| {
            board
                .state
                .read()
                .rows
                .iter()
                .filter(|row| row.needs_getting())
                .filter(|row| !view.engines.iter().any(|engine| engine.label == row.name))
                .map(|row| (row.id.clone(), row.name.clone()))
                .collect()
        })
        .unwrap_or_default();
    if view.engines.is_empty() && wanted.is_empty() {
        return rsx! { NoEngine { path, note: view.note.clone(), availability, onask } };
    }
    let engines: Vec<MenuItem<String>> = view
        .engines
        .iter()
        .map(|engine| MenuItem::new(engine.id.clone(), engine.label.clone()))
        .chain(
            wanted
                .iter()
                .map(|(id, name)| MenuItem::new(format!("{GET}{id}"), name.clone())),
        )
        .collect();
    let models: Vec<MenuItem<String>> = view
        .models()
        .iter()
        .flat_map(|model| pick_items(&model.word, &model.label, &model.gate))
        .collect();
    let (engine_at, model_at) = (path.clone(), path.clone());
    rsx! {
        FieldRow {
            label: TextLine::from(ENGINE_LABEL),
            availability,
            common: Common { id: Some(row_id("picker-engine", &path.0)), ..Common::default() },
            PopUpButton::<String> {
                items: engines,
                value: view.engine.clone(),
                title: Some(NOTHING_CHOSEN.to_owned()),
                size: ControlSize::Small,
                availability,
                onpick: move |engine: String| match (engine.strip_prefix(GET), board) {
                    (Some(id), Some(board)) => board.ask.call(EngineOfferInput::Chose(id.to_owned())),
                    _ => onask.call(PickerInput::Engine { path: engine_at.clone(), engine }),
                },
            }
        }
        if view.engine.is_some() {
            FieldRow {
                label: TextLine::from(MODEL_LABEL),
                availability,
                common: Common { id: Some(row_id("picker-local-model", &path.0)), ..Common::default() },
                PopUpButton::<String> {
                    items: models,
                    value: view.model.clone(),
                    title: Some(NOTHING_CHOSEN.to_owned()),
                    size: ControlSize::Small,
                    availability,
                    onpick: move |word| onask.call(PickerInput::Chose { path: model_at.clone(), word }),
                }
            }
        }
    }
}

/// Nothing on this computer is set up to run models: the row says so and adds the way in. The
/// module's own line says why when it has one, right under the row; otherwise the row's help does.
#[component]
fn NoEngine(
    path: KeyPath,
    note: Option<String>,
    availability: Availability,
    onask: EventHandler<PickerInput>,
) -> Element {
    let set_up = path.clone();
    rsx! {
        FieldRow {
            label: TextLine::from(NO_ENGINE_LABEL),
            help: note.is_none().then(|| TextLine::from(crate::copy::picker::gap(ChoiceGap::NoLocalEngine))),
            availability,
            common: Common { id: Some(row_id("picker-no-engine", &path.0)), ..Common::default() },
            Button {
                label: ADD,
                size: ControlSize::Small,
                availability,
                onclick: move |_| onask.call(PickerInput::SetUpEngine(set_up.clone())),
            }
        }
        if let Some(text) = note {
            NoteLine { text, kind: "no-engine" }
        }
    }
}
