//! "Local models": the models this computer runs itself, each with what it is doing, the memory
//! it takes while it is ready, and a Start or Stop. The list is read again each time the assistant
//! says an engine changed state. Nothing shows while the assistant is not running: opening
//! Settings must not start it.

use crate::copy;
use crate::key_row::row_id;
use crate::live::use_live;
use crate::picker::NoteLine;
use detent_client::Client;
use detent_model::{InFlight, Live, RunInput, RunRowView};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};
use ds::style::tokens::control_size::ControlSize;

/// The group's title.
const TITLE: &str = crate::copy::common::LOCAL_MODELS;

/// The group of models inferd runs itself; nothing when it lists none or is not there.
#[component]
pub(crate) fn RunningModels() -> Element {
    let running = use_live(Client::running);
    let Live::Ready(board) = (running.state)() else {
        return rsx! {};
    };
    let rows = board.rows();
    if rows.is_empty() {
        return rsx! {};
    }
    let onpress = EventHandler::new(move |model: String| {
        spawn(running.pane.send(RunInput::Act(model)));
    });
    rsx! {
        div { class: "running-models", "data-running": "list",
            FieldGroup { title: Some(TITLE.to_owned()),
                for row in rows {
                    ModelRow { key: "{row.model}", row: row.clone(), onpress }
                }
            }
        }
    }
}

/// One model: its name, what it is doing, the memory it takes, and its button, with the line
/// under it when it has one.
#[component]
fn ModelRow(row: RunRowView, onpress: EventHandler<String>) -> Element {
    let common = Common {
        data: DataName::parse("run")
            .map(|name| vec![DataAttr::new(name, row.standing.slug())])
            .unwrap_or_default(),
        ..Common::default()
    };
    let availability = match row.in_flight {
        InFlight::Yes => Availability::Busy,
        InFlight::No => Availability::Enabled,
    };
    let model = row.model.clone();
    let name = copy::running::model_name(&row.model);
    let action = copy::running::action(row.action);
    rsx! {
        FieldRow {
            label: TextLine::from(name),
            common,
            Label { text: copy::running::standing(&row.standing), role: LabelRole::Secondary, style: LabelStyle::Footnote }
            if let Some(mib) = row.memory_mib {
                Label { text: copy::running::memory_words(mib), role: LabelRole::Tertiary, style: LabelStyle::Footnote }
            }
            Button {
                label: action,
                size: ControlSize::Small,
                availability,
                common: Common {
                    id: Some(row_id("run", &row.model)),
                    aria_label: Some(format!("{action} {name}")),
                    ..Common::default()
                },
                onclick: move |_| onpress.call(model.clone()),
            }
        }
        if let Some(text) = row.note.clone() {
            NoteLine { text, kind: "model-run" }
        }
    }
}
