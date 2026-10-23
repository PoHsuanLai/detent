//! The running models' words and transitions.

use super::{
    InFlight, ModelRun, RunButton, RunEffect, RunInput, RunPane, RunStanding, memory_words,
    model_name, running_step,
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
fn each_state_has_its_plain_word_and_its_button() {
    let words = [
        (RunStanding::Stopped, "Stopped", RunButton::Start),
        (RunStanding::Starting, "Starting\u{2026}", RunButton::Stop),
        (RunStanding::Ready, "Ready", RunButton::Stop),
        (RunStanding::Stopping, "Stopping\u{2026}", RunButton::Stop),
        (
            RunStanding::Retrying,
            "Trying again\u{2026}",
            RunButton::Stop,
        ),
        (
            RunStanding::Failed(None),
            "Couldn\u{2019}t start",
            RunButton::Start,
        ),
    ];
    for (standing, word, button) in words {
        assert_eq!((standing.word(), standing.button()), (word, button));
    }
}

#[test]
fn memory_shows_only_while_ready_and_in_the_units_a_person_reads() {
    assert_eq!(memory_words(14_540), "14.2 GB");
    assert_eq!(memory_words(1024), "1.0 GB");
    assert_eq!(memory_words(512), "512 MB");
    let pane = listed(vec![
        run("local/qwen", RunStanding::Ready, 14_540),
        run("local/holo", RunStanding::Starting, 14_540),
    ]);
    let rows = board(&pane).rows();
    assert_eq!(rows[0].memory.as_deref(), Some("14.2 GB"));
    assert_eq!(rows[1].memory, None);
    assert_eq!(model_name("local/qwen"), "qwen");
    assert_eq!(model_name("local/org/qwen"), "org/qwen");
}

#[test]
fn a_press_asks_once_waits_and_ends_with_a_look() {
    let pane = listed(vec![run("local/qwen", RunStanding::Stopped, 0)]);
    let (pane, effects) = running_step(pane, RunInput::Press("local/qwen".to_owned()));
    assert_eq!(effects, [RunEffect::Start("local/qwen".to_owned())]);
    assert_eq!(board(&pane).rows()[0].in_flight, InFlight::Yes);
    let (pane, again) = running_step(pane, RunInput::Press("local/qwen".to_owned()));
    assert!(again.is_empty(), "a press that waits is not a second ask");
    let (pane, effects) = running_step(pane, RunInput::Answered("local/qwen".to_owned(), Ok(())));
    assert_eq!(effects, [RunEffect::Look]);
    assert_eq!(board(&pane).rows()[0].in_flight, InFlight::No);
}

#[test]
fn a_refusal_stays_under_its_row_until_the_next_press() {
    let pane = listed(vec![run("local/qwen", RunStanding::Stopped, 0)]);
    let (pane, _) = running_step(pane, RunInput::Press("local/qwen".to_owned()));
    let words = "needs 14.2 GB of graphics memory; 6.1 GB is free";
    let (pane, _) = running_step(
        pane,
        RunInput::Answered("local/qwen".to_owned(), Err(words.to_owned())),
    );
    assert_eq!(board(&pane).rows()[0].note.as_deref(), Some(words));
    let (pane, _) = running_step(pane, RunInput::Press("local/qwen".to_owned()));
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
fn a_list_that_cannot_be_read_leaves_the_pane_unavailable_and_a_press_does_nothing() {
    let (pane, _) = running_step(
        listed(vec![run("local/qwen", RunStanding::Ready, 1)]),
        RunInput::Found(Err(Absence::NotRunning)),
    );
    assert_eq!(pane, Live::Unavailable(Absence::NotRunning));
    let (_, effects) = running_step(pane, RunInput::Press("local/qwen".to_owned()));
    assert!(effects.is_empty());
}
