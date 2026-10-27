//! A pick sends the one change the compositor takes.

use super::*;
use crate::Absence;

#[test]
fn a_pick_sends_the_one_change_the_compositor_takes() {
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .answered("clipboard", "bbb", GRIM, "off")
        .unanswered("keys", "ccc", GRIM);
    let switch = path("privacy.screen.aaa");
    let forget = path("privacy.screen.aaa.forget");
    // `(case, the cell, what is picked, the effect)`.
    let cases: &[(&str, PrivacyPower, PowerStanding, Option<PrivacyEffect>)] = &[
        (
            "deny what was allowed",
            PrivacyPower::ScreenRecording,
            PowerStanding::Denied,
            Some(PrivacyEffect::Set {
                path: switch.clone(),
                to: PrivacySwitch::Off,
            }),
        ),
        (
            "ask again",
            PrivacyPower::ScreenRecording,
            PowerStanding::Ask,
            Some(PrivacyEffect::Forget(forget)),
        ),
        (
            "allow what was denied",
            PrivacyPower::Clipboard,
            PowerStanding::Allowed,
            Some(PrivacyEffect::Set {
                path: path("privacy.clipboard.bbb"),
                to: PrivacySwitch::On,
            }),
        ),
        (
            "pick what already stands",
            PrivacyPower::ScreenRecording,
            PowerStanding::Allowed,
            None,
        ),
        (
            "a power the compositor lists no row for",
            PrivacyPower::Windows,
            PowerStanding::Allowed,
            None,
        ),
        (
            "allow what nobody answered: the first answer",
            PrivacyPower::KeyboardInput,
            PowerStanding::Allowed,
            Some(PrivacyEffect::Set {
                path: path("privacy.keys.ccc"),
                to: PrivacySwitch::On,
            }),
        ),
        (
            "deny what nobody answered: the first answer",
            PrivacyPower::KeyboardInput,
            PowerStanding::Denied,
            Some(PrivacyEffect::Set {
                path: path("privacy.keys.ccc"),
                to: PrivacySwitch::Off,
            }),
        ),
        (
            "ask what already asks: nothing to forget",
            PrivacyPower::KeyboardInput,
            PowerStanding::Ask,
            None,
        ),
    ];
    for (case, power, to, want) in cases {
        let (after, effects) = pick(ready(&shown), GRIM, *power, *to);
        assert_eq!(effects, want.iter().cloned().collect::<Vec<_>>(), "{case}");
        assert_eq!(
            pending(&after).is_some(),
            want.is_some(),
            "{case}: a change in flight only when one was sent"
        );
    }
    assert_eq!(PrivacySwitch::On.word(), "on");
    assert_eq!(PrivacySwitch::Off.word(), "off");
}

#[test]
fn a_locked_cell_sends_nothing() {
    let shown = Publication::default().fixed("screen", "bbb", GRIM, "off");
    for to in PowerStanding::CHOICES {
        let (after, effects) = pick(ready(&shown), GRIM, PrivacyPower::ScreenRecording, to);
        assert!(effects.is_empty(), "{to:?}: {effects:?}");
        assert_eq!(pending(&after), None);
    }
}

#[test]
fn one_change_at_a_time_and_the_answer_reads_the_rows_again() {
    let shown = Publication::default().answered("screen", "aaa", GRIM, "on");
    let (busy, first) = pick(
        ready(&shown),
        GRIM,
        PrivacyPower::ScreenRecording,
        PowerStanding::Denied,
    );
    assert_eq!(first.len(), 1);
    let (busy, second) = pick(
        busy,
        GRIM,
        PrivacyPower::ScreenRecording,
        PowerStanding::Ask,
    );
    assert!(second.is_empty(), "while one is in flight: {second:?}");
    let (done, effects) = privacy_step(busy.clone(), PrivacyInput::Done);
    assert_eq!(effects, [PrivacyEffect::Reload]);
    assert_eq!(pending(&done), None);
    let (failed, effects) = privacy_step(busy, PrivacyInput::Failed("no".to_owned()));
    assert_eq!(
        effects,
        [PrivacyEffect::Tell("no".to_owned()), PrivacyEffect::Reload]
    );
    assert_eq!(pending(&failed), None);
}

#[test]
fn the_compositor_not_being_there_is_a_state_and_its_return_a_list() {
    let (gone, _) = privacy_step(
        PrivacyState::Looking,
        PrivacyInput::Found(Err(Absence::NotRunning)),
    );
    assert_eq!(gone, PrivacyState::Unavailable(Absence::NotRunning));
    let shown = Publication::default().answered("screen", "aaa", GRIM, "on");
    let (back, _) = privacy_step(gone, PrivacyInput::Found(Ok(shown.described())));
    assert_eq!(back, ready(&shown));
}
