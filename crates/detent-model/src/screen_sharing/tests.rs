//! The list's rows as the portal sends them, and the list's transitions.

use super::{
    APP_ID_MAX, ScreenGrant, ScreenSharingEffect, ScreenSharingInput, ScreenSharingState,
    ShareLength, screen_sharing_step,
};
use crate::Absence;
use crate::privacy::ProgramSubject;
use crate::updates::UnixSeconds;

const NOW: UnixSeconds = UnixSeconds(1_800_000_000);
const HOUR: i64 = 3600;
const DAY: i64 = 24 * HOUR;
const MEET: &str = "org.example.Meet";
const CHAT: &str = "org.example.Chat";

fn grant(app: &str, length: ShareLength, ago: i64) -> ScreenGrant {
    ScreenGrant {
        app: ProgramSubject(app.to_owned()),
        length,
        granted: UnixSeconds(NOW.0 - ago),
    }
}

fn subject(app: &str) -> ProgramSubject {
    ProgramSubject(app.to_owned())
}

fn ready(grants: Vec<ScreenGrant>) -> ScreenSharingState {
    screen_sharing_step(
        ScreenSharingState::Looking,
        ScreenSharingInput::Found(Ok(grants)),
    )
    .0
}

fn grants_of(state: &ScreenSharingState) -> Vec<&str> {
    match state {
        ScreenSharingState::Ready(ready) => ready.grants.iter().map(|g| g.app.0.as_str()).collect(),
        other => panic!("not ready: {other:?}"),
    }
}

#[test]
fn a_row_of_the_portal_is_a_grant_only_when_it_is_one() {
    let long = "a".repeat(APP_ID_MAX + 1);
    let longest = "a".repeat(APP_ID_MAX);
    let cases: &[(&str, &str, u32, u64, Option<ShareLength>)] = &[
        (
            "until taken back",
            MEET,
            2,
            100,
            Some(ShareLength::UntilRemoved),
        ),
        (
            "while it runs",
            MEET,
            1,
            100,
            Some(ShareLength::WhileRunning),
        ),
        (
            "an id as long as a file name",
            longest.as_str(),
            2,
            100,
            Some(ShareLength::UntilRemoved),
        ),
        (
            "an id longer than the portal takes back",
            long.as_str(),
            2,
            100,
            None,
        ),
        ("no id", "", 2, 100, None),
        ("a mode that remembers nothing", MEET, 0, 100, None),
        ("a mode the portal does not make", MEET, 3, 100, None),
        ("a date out of range", MEET, 2, u64::MAX, None),
    ];
    for (name, app, mode, granted, want) in cases {
        let got = ScreenGrant::of_wire(app, *mode, *granted);
        assert_eq!(got.map(|g| g.length), *want, "{name}");
    }
    let one = ScreenGrant::of_wire(MEET, 2, 1_799_999_000).expect("a grant");
    assert_eq!(one.app, subject(MEET));
    assert_eq!(one.granted, UnixSeconds(1_799_999_000));
}

#[test]
fn the_list_is_shown_with_the_app_allowed_last_first() {
    let state = ready(vec![
        grant(MEET, ShareLength::UntilRemoved, 5 * DAY),
        grant(CHAT, ShareLength::UntilRemoved, DAY),
        grant("org.example.Aaa", ShareLength::WhileRunning, DAY),
    ]);
    assert_eq!(
        grants_of(&state),
        ["org.example.Aaa", CHAT, MEET],
        "newest first, and by id when two were allowed together"
    );
}

#[test]
fn remove_asks_the_portal_once_and_the_answer_reads_the_list_again() {
    let state = ready(vec![
        grant(MEET, ShareLength::UntilRemoved, DAY),
        grant(CHAT, ShareLength::UntilRemoved, 2 * DAY),
    ]);
    let (asked, effects) = screen_sharing_step(state, ScreenSharingInput::Remove(subject(MEET)));
    assert_eq!(effects, [ScreenSharingEffect::Revoke(subject(MEET))]);
    // While one is under way, a second press asks nothing.
    let (asked, effects) = screen_sharing_step(asked, ScreenSharingInput::Remove(subject(CHAT)));
    assert!(effects.is_empty());
    let (done, effects) = screen_sharing_step(asked, ScreenSharingInput::Removed(Ok(())));
    assert_eq!(effects, [ScreenSharingEffect::Reload]);
    let (listed, _) = screen_sharing_step(
        done,
        ScreenSharingInput::Found(Ok(vec![grant(CHAT, ShareLength::UntilRemoved, 2 * DAY)])),
    );
    assert_eq!(grants_of(&listed), [CHAT]);
    // And the next press is heard again.
    let (_, effects) = screen_sharing_step(listed, ScreenSharingInput::Remove(subject(CHAT)));
    assert_eq!(effects, [ScreenSharingEffect::Revoke(subject(CHAT))]);
}

#[test]
fn an_app_that_is_not_listed_is_not_asked_about() {
    let state = ready(vec![grant(MEET, ShareLength::UntilRemoved, DAY)]);
    let (same, effects) = screen_sharing_step(
        state.clone(),
        ScreenSharingInput::Remove(subject("org.example.Other")),
    );
    assert!(effects.is_empty());
    assert_eq!(same, state);
}

#[test]
fn a_refusal_is_told_and_the_list_is_read_again() {
    let state = ready(vec![grant(MEET, ShareLength::UntilRemoved, DAY)]);
    let (asked, _) = screen_sharing_step(state, ScreenSharingInput::Remove(subject(MEET)));
    let words = "Screen sharing could not be changed.".to_owned();
    let (_, effects) = screen_sharing_step(asked, ScreenSharingInput::Removed(Err(words.clone())));
    assert_eq!(
        effects,
        [
            ScreenSharingEffect::Tell(words),
            ScreenSharingEffect::Reload
        ]
    );
}

#[test]
fn the_portal_going_away_is_said_and_coming_back_lists_again() {
    let state = ready(vec![grant(MEET, ShareLength::UntilRemoved, DAY)]);
    let (gone, _) = screen_sharing_step(state, ScreenSharingInput::Found(Err(Absence::NotRunning)));
    assert_eq!(gone, ScreenSharingState::Unavailable(Absence::NotRunning));
    let (_, effects) = screen_sharing_step(gone.clone(), ScreenSharingInput::Remove(subject(MEET)));
    assert!(effects.is_empty(), "nothing to remove while it is gone");
    let back = screen_sharing_step(gone, ScreenSharingInput::Found(Ok(vec![]))).0;
    assert_eq!(grants_of(&back), Vec::<&str>::new());
}
