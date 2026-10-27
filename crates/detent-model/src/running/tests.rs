//! The running models' facts and transitions.

use super::{
    InFlight, ModelRun, RunAction, RunEffect, RunInput, RunPane, RunStanding, running_step,
};
use crate::{Absence, Live};

fn run(model: &str, standing: RunStanding, memory_mib: u32) -> ModelRun {
    ModelRun {
        model: model.to_owned(),
        standing,
        memory_mib,
    }
}

fn listed(rows: Vec<ModelRun>) -> RunPane {
    running_step(Live::Looking, RunInput::Found(Ok(rows))).0
}

fn board(pane: &RunPane) -> &super::RunBoard {
    match pane {
        Live::Ready(board) => board,
        other => panic!("not listed: {other:?}"),
    }
}

#[test]
fn each_state_has_its_action() {
    let actions = [
        (RunStanding::Stopped, RunAction::Start),
        (RunStanding::Starting, RunAction::Stop),
        (RunStanding::Ready, RunAction::Stop),
        (RunStanding::Stopping, RunAction::Stop),
        (RunStanding::Retrying, RunAction::Stop),
        (RunStanding::Failed(None), RunAction::Start),
    ];
    for (standing, action) in actions {
        assert_eq!(standing.action(), action, "{standing:?}");
    }
}

#[test]
fn memory_shows_only_while_ready() {
    let pane = listed(vec![
        run("local/qwen", RunStanding::Ready, 14_540),
        run("local/holo", RunStanding::Starting, 14_540),
        run("local/idle", RunStanding::Ready, 0),
    ]);
    let rows = board(&pane).rows();
    assert_eq!(rows[0].memory_mib, Some(14_540));
    assert_eq!(rows[1].memory_mib, None);
    assert_eq!(
        rows[2].memory_mib, None,
        "a model that takes none says none"
    );
}

#[test]
fn an_ask_is_made_once_waits_and_ends_with_a_look() {
    let pane = listed(vec![run("local/qwen", RunStanding::Stopped, 0)]);
    let (pane, effects) = running_step(pane, RunInput::Act("local/qwen".to_owned()));
    assert_eq!(effects, [RunEffect::Start("local/qwen".to_owned())]);
    assert_eq!(board(&pane).rows()[0].in_flight, InFlight::Yes);
    let (pane, again) = running_step(pane, RunInput::Act("local/qwen".to_owned()));
    assert!(again.is_empty(), "an ask that waits is not a second ask");
    let (pane, effects) = running_step(pane, RunInput::Answered("local/qwen".to_owned(), Ok(())));
    assert_eq!(effects, [RunEffect::Look]);
    assert_eq!(board(&pane).rows()[0].in_flight, InFlight::No);
}

#[test]
fn a_refusal_stays_under_its_row_until_the_next_ask() {
    let pane = listed(vec![run("local/qwen", RunStanding::Stopped, 0)]);
    let (pane, _) = running_step(pane, RunInput::Act("local/qwen".to_owned()));
    let words = "needs 14.2 GB of graphics memory; 6.1 GB is free";
    let (pane, _) = running_step(
        pane,
        RunInput::Answered("local/qwen".to_owned(), Err(words.to_owned())),
    );
    assert_eq!(board(&pane).rows()[0].note.as_deref(), Some(words));
    let (pane, _) = running_step(pane, RunInput::Act("local/qwen".to_owned()));
    assert_eq!(board(&pane).rows()[0].note, None);
}

#[test]
fn a_failed_model_shows_its_reason_and_a_stopped_one_in_flight_shows_progress() {
    let pane = listed(vec![
        run(
            "local/qwen",
            RunStanding::Failed(Some("no room".to_owned())),
            0,
        ),
        run("local/holo", RunStanding::Stopping, 0),
    ]);
    let rows = board(&pane).rows();
    assert_eq!(rows[0].note.as_deref(), Some("no room"));
    assert_eq!(rows[1].in_flight, InFlight::Yes);
}

#[test]
fn a_list_that_cannot_be_read_leaves_the_pane_unavailable_and_an_ask_does_nothing() {
    let (pane, _) = running_step(
        listed(vec![run("local/qwen", RunStanding::Ready, 1)]),
        RunInput::Found(Err(Absence::NotRunning)),
    );
    assert_eq!(pane, Live::Unavailable(Absence::NotRunning));
    let (_, effects) = running_step(pane, RunInput::Act("local/qwen".to_owned()));
    assert!(effects.is_empty());
}
