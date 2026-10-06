use super::*;
use crate::accounts::{AccountKey, AccountStatus};

fn runtime(key: &str, status: AccountStatus) -> Runtime {
    Runtime {
        key: AccountKey(key.to_owned()),
        label: key.to_owned(),
        status,
    }
}

fn found() -> FirstRun {
    first_run_step(
        FirstRun::default(),
        FirstRunInput::Found(vec![
            runtime("ollama", AccountStatus::Ok),
            runtime("lm-studio", AccountStatus::Offline),
        ]),
    )
    .0
}

fn key(text: &str) -> Option<AccountKey> {
    Some(AccountKey(text.to_owned()))
}

#[test]
fn ai_stays_on_this_computer_until_the_person_says_otherwise() {
    let state = FirstRun::default();
    assert_eq!(state.local_only, LocalOnly::On);
    let (state, effects) = first_run_step(state, FirstRunInput::Finish);
    assert_eq!(
        effects,
        [FirstRunEffect::Save {
            local_only: LocalOnly::On,
            runtime: None
        }]
    );
    assert_eq!(state.stage, Stage::Saving);
}

#[test]
fn only_a_running_runtime_can_be_picked() {
    let (state, _) = first_run_step(found(), FirstRunInput::Picked(key("lm-studio")));
    assert_eq!(state.picked, None, "a stopped runtime cannot be picked");
    let (state, _) = first_run_step(state, FirstRunInput::Picked(key("nobody")));
    assert_eq!(state.picked, None);
    let (state, _) = first_run_step(state, FirstRunInput::Picked(key("ollama")));
    assert_eq!(state.picked, key("ollama"));
    let (state, effects) = first_run_step(state, FirstRunInput::Finish);
    assert_eq!(
        effects,
        [FirstRunEffect::Save {
            local_only: LocalOnly::On,
            runtime: key("ollama")
        }]
    );
    assert_eq!(state.stage, Stage::Saving);
}

#[test]
fn a_picked_runtime_that_stops_is_no_longer_picked() {
    let (state, _) = first_run_step(found(), FirstRunInput::Picked(key("ollama")));
    let (state, _) = first_run_step(
        state,
        FirstRunInput::Found(vec![runtime("ollama", AccountStatus::Offline)]),
    );
    assert_eq!(state.picked, None);
}

#[test]
fn a_failed_save_asks_again_and_a_saved_one_closes() {
    let (saving, _) = first_run_step(found(), FirstRunInput::Finish);
    let (asking, effects) = first_run_step(saving.clone(), FirstRunInput::Failed("no".to_owned()));
    assert_eq!(effects, [FirstRunEffect::Tell("no".to_owned())]);
    assert_eq!(asking.stage, Stage::Asking);
    let (done, effects) = first_run_step(saving, FirstRunInput::Saved);
    assert_eq!(effects, [FirstRunEffect::Close]);
    assert_eq!(done.stage, Stage::Done);
}

#[test]
fn nothing_changes_while_saving_and_done_is_pressed_once() {
    let (saving, _) = first_run_step(found(), FirstRunInput::Finish);
    for input in [
        FirstRunInput::Kept(LocalOnly::Off),
        FirstRunInput::Picked(key("ollama")),
        FirstRunInput::Finish,
    ] {
        let (next, effects) = first_run_step(saving.clone(), input);
        assert_eq!(next, saving);
        assert!(effects.is_empty());
    }
}
