//! "Local models": the models this computer runs itself, each with what it is doing, the memory
//! it takes while it is ready, and a Start or Stop. The list is read again each time the assistant
//! says an engine changed state. Nothing shows while the assistant is not running: opening
//! Settings must not start it.

use crate::copy;
use crate::env::Env;
use crate::key_row::row_id;
use crate::picker::NoteLine;
use detent_model::{
    InFlight, Live, RunAction, RunEffect, RunInput, RunPane, RunRowView, running_step,
};
use detent_system::bus::{BusSource, Daemon, connect, next_owner, owner_changes};
use detent_system::running::RunningLink;
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};
use ds::style::tokens::control_size::ControlSize;
use std::sync::Arc;

/// The group's title.
const TITLE: &str = "Local models";

/// Steps the pane on `input` and does what that asks; what each ask ends with is fed back in.
fn drive(mut pane: Signal<RunPane>, link: Arc<dyn RunningLink>, input: RunInput) {
    let (next, effects) = running_step(pane.peek().clone(), input);
    pane.set(next);
    for effect in effects {
        let link = link.clone();
        spawn(async move {
            match effect {
                RunEffect::Look => {
                    let found = link.read().await;
                    drive(pane, link, RunInput::Found(found));
                }
                RunEffect::Start(model) => {
                    let ended = link.act(model.clone(), RunAction::Start).await;
                    drive(pane, link, RunInput::Answered(model, ended));
                }
                RunEffect::Stop(model) => {
                    let ended = link.act(model.clone(), RunAction::Stop).await;
                    drive(pane, link, RunInput::Answered(model, ended));
                }
            }
        });
    }
}

/// Reads the list, then reads it again each time a model changes state or inferd comes back.
async fn follow(pane: Signal<RunPane>, bus: BusSource, link: Arc<dyn RunningLink>) {
    let connection = match connect(&bus).await {
        Ok(connection) => connection,
        Err(absence) => return drive(pane, link, RunInput::Found(Err(absence))),
    };
    let Ok(mut owners) = owner_changes(&connection, &Daemon::inferd()).await else {
        return;
    };
    loop {
        // Subscribed before the first read, so a change in between is not lost.
        let watch = link.changes().await;
        drive(pane, link.clone(), RunInput::Found(link.read().await));
        if let Ok(mut watch) = watch {
            loop {
                tokio::select! {
                    changed = watch.recv() => match changed {
                        Some(()) => drive(pane, link.clone(), RunInput::Found(link.read().await)),
                        None => break,
                    },
                    owner = next_owner(&mut owners) => match owner {
                        Some(()) => break,
                        None => return,
                    },
                }
            }
            continue;
        }
        if next_owner(&mut owners).await.is_none() {
            return;
        }
    }
}

/// The group of models inferd runs itself; nothing when it lists none or is not there.
#[component]
pub(crate) fn RunningModels() -> Element {
    let env = use_context::<Env>();
    let pane = use_signal(RunPane::default);
    use_future({
        let (bus, link) = (env.bus.clone(), env.running.clone());
        move || follow(pane, bus.clone(), link.clone())
    });
    let Live::Ready(board) = pane() else {
        return rsx! {};
    };
    let rows = board.rows();
    if rows.is_empty() {
        return rsx! {};
    }
    let link = env.running.clone();
    let onpress = EventHandler::new(move |model: String| {
        drive(pane, link.clone(), RunInput::Act(model));
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
