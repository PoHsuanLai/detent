//! What an action may ask of the service: only what the standing allows.

use super::super::{
    NotesState, UpdatePhase, UpdateStatus, UpdatesEffect, UpdatesInput, UpdatesState, updates_step,
};
use super::{newer, offered, ready, status, version};
use crate::Absence;

#[test]
fn a_press_asks_the_service_only_when_the_standing_allows_it() {
    // `(case, the status, the press, what is asked)`.
    let cases: Vec<(&str, UpdateStatus, UpdatesInput, Vec<UpdatesEffect>)> = vec![
        (
            "check when idle",
            status(UpdatePhase::Idle),
            UpdatesInput::Check,
            vec![UpdatesEffect::Check],
        ),
        (
            "check again when an update waits",
            offered(UpdatePhase::Available),
            UpdatesInput::Check,
            vec![UpdatesEffect::Check],
        ),
        (
            "no check while one is running",
            status(UpdatePhase::Checking),
            UpdatesInput::Check,
            vec![],
        ),
        (
            "no check while installing",
            offered(UpdatePhase::Installing),
            UpdatesInput::Check,
            vec![],
        ),
        (
            "update now installs",
            offered(UpdatePhase::Available),
            UpdatesInput::UpdateNow,
            vec![UpdatesEffect::Install(version("0.4.0"))],
        ),
        (
            "update now with nothing on offer asks nothing",
            status(UpdatePhase::Available),
            UpdatesInput::UpdateNow,
            vec![],
        ),
        (
            "update now while downloading asks nothing",
            offered(UpdatePhase::Downloading),
            UpdatesInput::UpdateNow,
            vec![],
        ),
        (
            "try again installs what was on offer",
            offered(UpdatePhase::Failed),
            UpdatesInput::TryAgain,
            vec![UpdatesEffect::Install(version("0.4.0"))],
        ),
        (
            "try again looks again when nothing was on offer",
            status(UpdatePhase::Failed),
            UpdatesInput::TryAgain,
            vec![UpdatesEffect::Check],
        ),
        (
            "try again when nothing failed asks nothing",
            status(UpdatePhase::Idle),
            UpdatesInput::TryAgain,
            vec![],
        ),
        (
            "log out once installed",
            status(UpdatePhase::Installed),
            UpdatesInput::LogOut,
            vec![UpdatesEffect::LogOut],
        ),
        (
            "no log out before it is installed",
            offered(UpdatePhase::Available),
            UpdatesInput::LogOut,
            vec![],
        ),
        (
            "a refusal is told in the service's words",
            status(UpdatePhase::Idle),
            UpdatesInput::Refused("Another update is in progress.".to_owned()),
            vec![UpdatesEffect::Tell(
                "Another update is in progress.".to_owned(),
            )],
        ),
    ];
    for (case, status, press, want) in cases {
        let (_, effects) = updates_step(ready(status), press);
        assert_eq!(effects, want, "{case}");
    }
}

#[test]
fn a_page_without_a_service_asks_nothing() {
    for state in [
        UpdatesState::Looking,
        UpdatesState::Unavailable(Absence::NotRunning),
    ] {
        for press in [
            UpdatesInput::Check,
            UpdatesInput::UpdateNow,
            UpdatesInput::TryAgain,
            UpdatesInput::LogOut,
        ] {
            let (after, effects) = updates_step(state.clone(), press);
            assert_eq!((after, effects), (state.clone(), vec![]));
        }
    }
}

#[test]
fn an_install_names_the_version_the_page_shows_and_never_one_that_is_not_shown() {
    for (press, phase) in [
        (UpdatesInput::UpdateNow, UpdatePhase::Available),
        (UpdatesInput::TryAgain, UpdatePhase::Failed),
    ] {
        let (_, effects) = updates_step(ready(newer(phase)), press);
        assert_eq!(effects, vec![UpdatesEffect::Install(version("0.5.0"))]);
    }
    // With no version on the page there is nothing to name: no standing turns a press into an
    // install.
    for phase in [
        UpdatePhase::Idle,
        UpdatePhase::Checking,
        UpdatePhase::Available,
        UpdatePhase::Downloading,
        UpdatePhase::Ready,
        UpdatePhase::Installing,
        UpdatePhase::Installed,
        UpdatePhase::Failed,
    ] {
        for press in [UpdatesInput::UpdateNow, UpdatesInput::TryAgain] {
            let (_, effects) = updates_step(ready(status(phase)), press);
            assert!(
                effects
                    .iter()
                    .all(|effect| !matches!(effect, UpdatesEffect::Install(_))),
                "{phase:?}: {effects:?}"
            );
        }
    }
}

#[test]
fn what_the_service_says_is_taken_and_a_silent_service_is_unavailable() {
    let (state, _) = updates_step(
        UpdatesState::Looking,
        UpdatesInput::Heard(Ok(status(UpdatePhase::Idle))),
    );
    assert_eq!(state, ready(status(UpdatePhase::Idle)));
    let (state, _) = updates_step(state, UpdatesInput::Heard(Err(Absence::NotRunning)));
    assert_eq!(state, UpdatesState::Unavailable(Absence::NotRunning));
    let (state, _) = updates_step(state, UpdatesInput::Heard(Ok(status(UpdatePhase::Idle))));
    assert_eq!(state, ready(status(UpdatePhase::Idle)), "it came back");
}

#[test]
fn the_notes_open_while_an_update_waits_and_close_when_it_does_not() {
    let (state, effects) = updates_step(
        ready(offered(UpdatePhase::Available)),
        UpdatesInput::MoreInfo,
    );
    assert_eq!(effects, vec![]);
    let UpdatesState::Ready(open) = &state else {
        panic!("not ready: {state:?}");
    };
    assert_eq!(open.info, NotesState::Open);

    // The same update, heard again: the sheet stays.
    let (state, _) = updates_step(
        state,
        UpdatesInput::Heard(Ok(offered(UpdatePhase::Available))),
    );
    assert!(matches!(&state, UpdatesState::Ready(r) if r.info == NotesState::Open));

    // The download starts behind the sheet: it goes.
    let (after, _) = updates_step(
        state.clone(),
        UpdatesInput::Heard(Ok(offered(UpdatePhase::Downloading))),
    );
    assert!(matches!(&after, UpdatesState::Ready(r) if r.info == NotesState::Closed));

    let (closed, _) = updates_step(state, UpdatesInput::CloseInfo);
    assert!(matches!(&closed, UpdatesState::Ready(r) if r.info == NotesState::Closed));

    let (none, _) = updates_step(ready(status(UpdatePhase::Idle)), UpdatesInput::MoreInfo);
    assert!(
        matches!(&none, UpdatesState::Ready(r) if r.info == NotesState::Closed),
        "nothing to read about"
    );
}
