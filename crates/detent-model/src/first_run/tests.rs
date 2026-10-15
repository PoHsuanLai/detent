use super::*;

#[test]
fn ai_stays_on_this_computer_until_the_person_says_otherwise() {
    let state = FirstRun::default();
    assert_eq!(state.local_only, LocalOnly::On);
    let (state, effects) = first_run_step(state, FirstRunInput::Finish);
    assert_eq!(
        effects,
        [FirstRunEffect::Save {
            local_only: LocalOnly::On
        }]
    );
    assert_eq!(state.stage, Stage::Saving);
    let (off, _) = first_run_step(FirstRun::default(), FirstRunInput::Kept(LocalOnly::Off));
    let (_, effects) = first_run_step(off, FirstRunInput::Finish);
    assert_eq!(
        effects,
        [FirstRunEffect::Save {
            local_only: LocalOnly::Off
        }]
    );
}

#[test]
fn a_failed_save_asks_again_and_a_saved_one_closes() {
    let (saving, _) = first_run_step(FirstRun::default(), FirstRunInput::Finish);
    let (asking, effects) = first_run_step(saving.clone(), FirstRunInput::Failed("no".to_owned()));
    assert_eq!(effects, [FirstRunEffect::Tell("no".to_owned())]);
    assert_eq!(asking.stage, Stage::Asking);
    let (done, effects) = first_run_step(saving, FirstRunInput::Saved);
    assert_eq!(effects, [FirstRunEffect::Close]);
    assert_eq!(done.stage, Stage::Done);
}

#[test]
fn nothing_changes_while_saving_and_done_is_pressed_once() {
    let (saving, _) = first_run_step(FirstRun::default(), FirstRunInput::Finish);
    for input in [FirstRunInput::Kept(LocalOnly::Off), FirstRunInput::Finish] {
        let (next, effects) = first_run_step(saving.clone(), input);
        assert_eq!(next, saving);
        assert!(effects.is_empty());
    }
}
