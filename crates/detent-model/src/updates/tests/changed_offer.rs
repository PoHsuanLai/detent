//! An offer that changed under the person: the note, and when it stays.

use super::super::{
    ByteCount, ProblemKind, UpdateAction, UpdatePhase, UpdateStanding, UpdateStatus, UpdatesEffect,
    UpdatesInput, UpdatesNotice, UpdatesState, updates_step,
};
use super::{newer, offered, page_of, ready, status, version};

/// What the service says when an install named a version that is not on offer any more; the page
/// does not read it to know that (the kind says so), it only shows it.
const OVERTAKEN: &str = "A different update is on offer now. Look at it again before installing.";

fn notice_of(state: &UpdatesState) -> UpdatesNotice {
    match state {
        UpdatesState::Ready(ready) => ready.notice,
        other => panic!("not ready: {other:?}"),
    }
}

#[test]
fn an_offer_that_changed_between_showing_and_pressing_is_shown_again_with_a_note() {
    // The page showed 0.4.0; the person pressed; the service named 0.5.0 as on offer.
    let (state, _) = updates_step(
        ready(offered(UpdatePhase::Available)),
        UpdatesInput::Heard(Ok(newer(UpdatePhase::Available))),
    );
    assert_eq!(
        notice_of(&state),
        UpdatesNotice::Nothing,
        "a new offer alone says nothing"
    );
    let (state, effects) = updates_step(state, UpdatesInput::VersionChanged);
    assert_eq!(effects, vec![]);
    let page = page_of(&state);
    assert_eq!(page.version.as_deref(), Some("0.5.0"));
    assert_eq!(page.notice, UpdatesNotice::NewerVersion);

    // Heard again with the same offer: the note stays. Pressing Update Now names what is shown
    // and puts the note away.
    let (state, _) = updates_step(
        state,
        UpdatesInput::Heard(Ok(newer(UpdatePhase::Available))),
    );
    assert_eq!(notice_of(&state), UpdatesNotice::NewerVersion);
    let (after, effects) = updates_step(state.clone(), UpdatesInput::UpdateNow);
    assert_eq!(effects, vec![UpdatesEffect::Install(version("0.5.0"))]);
    assert_eq!(notice_of(&after), UpdatesNotice::Nothing);

    // The offer goes away: so does the note.
    let (gone, _) = updates_step(state, UpdatesInput::Heard(Ok(status(UpdatePhase::Idle))));
    assert_eq!(notice_of(&gone), UpdatesNotice::Nothing);
    assert_eq!(page_of(&gone).notice, UpdatesNotice::Nothing);
}

#[test]
fn a_changed_offer_with_nothing_on_offer_leaves_no_note() {
    let (state, _) = updates_step(
        ready(status(UpdatePhase::Idle)),
        UpdatesInput::VersionChanged,
    );
    assert_eq!(state, ready(status(UpdatePhase::Idle)));
}

#[test]
fn a_failure_that_says_the_offer_changed_is_an_offer_with_the_note() {
    let overtaken = UpdateStatus {
        problem: Some(OVERTAKEN.to_owned()),
        problem_kind: ProblemKind::VersionChanged,
        ..newer(UpdatePhase::Failed)
    };
    let (state, effects) = updates_step(UpdatesState::Looking, UpdatesInput::Heard(Ok(overtaken)));
    assert_eq!(effects, vec![]);
    let page = page_of(&state);
    assert_eq!(page.version.as_deref(), Some("0.5.0"));
    assert_eq!(
        page.standing,
        UpdateStanding::Available(ByteCount(45_600_000))
    );
    assert_eq!(
        page.actions,
        vec![UpdateAction::MoreInfo, UpdateAction::UpdateNow]
    );
    assert_eq!(page.notice, UpdatesNotice::NewerVersion);

    // With nothing on offer, or from a service whose kind is not known (an older one), it is a
    // failure like any other, in the service's words.
    for failed in [
        UpdateStatus {
            problem_kind: ProblemKind::VersionChanged,
            ..status(UpdatePhase::Failed)
        },
        newer(UpdatePhase::Failed),
    ] {
        let failed = UpdateStatus {
            problem: Some(OVERTAKEN.to_owned()),
            ..failed
        };
        let page = page_of(&ready(failed));
        assert_eq!(
            page.standing,
            UpdateStanding::Failed(Some(OVERTAKEN.to_owned()))
        );
        assert_eq!(page.actions, vec![UpdateAction::TryAgain]);
        assert_eq!(page.notice, UpdatesNotice::Nothing);
    }
}
