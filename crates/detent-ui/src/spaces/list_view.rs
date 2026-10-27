//! The Spaces list: one row per Space with Remove, or why the list is not there.

use super::sheet_view::{RemovalSheetView, SheetHandlers};
use crate::copy::absence::{Subject, absence as absence_words};
use crate::icons::tile_of;
use crate::live::use_live;
use crate::system::Absent;
use detent_client::Client;
use detent_model::{
    PageKind, RemovalStage, SpaceKey, SpaceRow, SpacesInput, SpacesReady, SpacesState,
};
use dioxus::prelude::*;
use ds::components::controls::button_model::ButtonRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// The Spaces the desktop keeps, with a Remove on each.
#[component]
pub(crate) fn SpacesPane() -> Element {
    let spaces = use_live(Client::spaces);
    let state = spaces.state;
    let drive = EventHandler::new(move |input: SpacesInput| {
        spawn(spaces.pane.send(input));
    });

    match state() {
        SpacesState::Looking => rsx! {},
        SpacesState::Unavailable(absence) => rsx! {
            div { class: "spaces", "data-spaces": "unavailable",
                Absent {
                    face: Some(tile_of(PageKind::Spaces)),
                    title: crate::copy::spaces::SPACES,
                    words: absence_words(Subject::Spaces, absence),
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
                    title: crate::copy::spaces::NO_SPACES,
                    description: crate::copy::spaces::NO_SPACES_HINT,
                }
            } else {
                FieldGroup { title: Some(crate::copy::spaces::SPACES.to_owned()),
                    for row in ready.rows.iter().cloned() {
                        SpaceLine { key: "{row.id.0}", row, availability: busy, drive }
                    }
                }
            }
            if let RemovalStage::Asking(ask) = ready.stage.clone() {
                RemovalSheetView {
                    ask,
                    on: SheetHandlers {
                        changed: EventHandler::new(move |change| drive.call(SpacesInput::Answer(change))),
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
                    aria_label: Some(crate::copy::spaces::remove(&row.name)),
                    ..Common::default()
                },
                label: crate::copy::common::REMOVE,
                role: ButtonRole::Destructive,
                size: ControlSize::Small,
                availability,
                onclick: move |_| drive.call(SpacesInput::AskRemove(key.clone())),
            }
        }
    }
}
