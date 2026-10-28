//! The removal question and its wire, and the list's transitions.

use super::removal::{RemovalInput, needs_memory};
use super::{
    RemovalAsk, RemovalStage, SpaceKey, SpaceRow, SpacesEffect, SpacesInput, SpacesState,
    spaces_step,
};
use almanac_core::{HistoryFate, MemoryFate, Removal};

fn trip() -> RemovalAsk {
    RemovalAsk::of(SpaceKey("space-1".to_owned()), "Trip".to_owned())
}

fn ready() -> SpacesState {
    let rows = vec![
        SpaceRow {
            id: SpaceKey("space-1".to_owned()),
            name: "Trip".to_owned(),
        },
        SpaceRow {
            id: SpaceKey("space-2".to_owned()),
            name: "Work".to_owned(),
        },
    ];
    spaces_step(SpacesState::Looking, SpacesInput::Found(Ok(rows))).0
}

fn asked() -> SpacesState {
    spaces_step(
        ready(),
        SpacesInput::AskRemove(SpaceKey("space-1".to_owned())),
    )
    .0
}

#[test]
fn the_question_opens_safe_flips_and_makes_remove_destructive_by_either_delete_choice() {
    // It opens on the choices nothing is lost by.
    let ask = trip();
    assert_eq!(ask.name, "Trip");
    assert_eq!(ask.memories, MemoryFate::MoveToApps);
    assert_eq!(ask.history, HistoryFate::Keep);
    assert!(!ask.destroys());
    assert_eq!(ask.removal(), Removal::KEEP_ALL);
    // The flip changes what becomes of the memories, and back.
    let flipped = trip().after(RemovalInput::FlipMemories);
    assert_eq!(flipped.memories, MemoryFate::Delete);
    assert!(flipped.destroys());
    // Each delete choice alone makes Remove destructive.
    let history = trip().after(RemovalInput::History(HistoryFate::Delete));
    assert!(history.destroys());
    assert_eq!(history.memories, MemoryFate::MoveToApps);
    assert!(flipped.destroys());
    assert_eq!(flipped.history, HistoryFate::Keep);
    assert_eq!(flipped.after(RemovalInput::FlipMemories), trip());
}

#[test]
fn the_choices_send_the_json_memory_reads() {
    let both = trip()
        .after(RemovalInput::FlipMemories)
        .after(RemovalInput::History(HistoryFate::Delete));
    assert_eq!(
        serde_json::to_string(&both.removal()).expect("a removal is JSON"),
        r#"{"memories":"delete","history":"delete"}"#
    );
    assert_eq!(
        serde_json::to_string(&trip().removal()).expect("a removal is JSON"),
        r#"{"memories":"move_to_apps","history":"keep"}"#
    );
}

#[test]
fn asking_opens_the_question_and_cancel_sends_nothing() {
    let SpacesState::Ready(ready) = asked() else {
        panic!("the list is there");
    };
    assert_eq!(ready.stage, RemovalStage::Asking(trip()));
    let (after, effects) = spaces_step(SpacesState::Ready(ready), SpacesInput::Cancel);
    assert!(effects.is_empty(), "Cancel sends nothing");
    let SpacesState::Ready(after) = after else {
        panic!("still the list");
    };
    assert_eq!(after.stage, RemovalStage::Idle);
}

#[test]
fn confirming_sends_the_chosen_removal_for_that_space_once() {
    let changed = spaces_step(
        asked(),
        SpacesInput::Answer(RemovalInput::History(HistoryFate::Delete)),
    )
    .0;
    let (after, effects) = spaces_step(changed, SpacesInput::Confirm);
    assert_eq!(
        effects,
        [SpacesEffect::Remove {
            space: SpaceKey("space-1".to_owned()),
            removal: Removal {
                memories: MemoryFate::MoveToApps,
                history: HistoryFate::Delete,
            },
        }]
    );
    let (_, again) = spaces_step(after, SpacesInput::Confirm);
    assert!(
        again.is_empty(),
        "the question is gone, so a second press sends nothing"
    );
}

#[test]
fn nothing_else_is_asked_while_a_removal_is_under_way() {
    let (under_way, _) = spaces_step(asked(), SpacesInput::Confirm);
    let (after, _) = spaces_step(
        under_way,
        SpacesInput::AskRemove(SpaceKey("space-2".to_owned())),
    );
    let SpacesState::Ready(after) = after else {
        panic!("the list is there");
    };
    assert_eq!(
        after.stage,
        RemovalStage::Removing(SpaceKey("space-1".to_owned())),
        "the first removal goes on and no second question opens"
    );
}

#[test]
fn a_failed_removal_is_told_and_the_list_read_again() {
    let (under_way, _) = spaces_step(asked(), SpacesInput::Confirm);
    let (after, effects) = spaces_step(under_way, SpacesInput::Removed(Err("no".to_owned())));
    assert_eq!(
        effects,
        [SpacesEffect::Tell("no".to_owned()), SpacesEffect::Reload]
    );
    let SpacesState::Ready(after) = after else {
        panic!("the list is there");
    };
    assert_eq!(after.stage, RemovalStage::Idle);
}

#[test]
fn a_delete_choice_waits_for_memory_and_a_keeping_one_does_not() {
    // `(case, memories, history, whether it waits for memory)`.
    const CASES: &[(&str, MemoryFate, HistoryFate, bool)] = &[
        (
            "keep everything",
            MemoryFate::MoveToApps,
            HistoryFate::Keep,
            false,
        ),
        (
            "delete the memories",
            MemoryFate::Delete,
            HistoryFate::Keep,
            true,
        ),
        (
            "delete the history",
            MemoryFate::MoveToApps,
            HistoryFate::Delete,
            true,
        ),
        ("delete both", MemoryFate::Delete, HistoryFate::Delete, true),
    ];
    for (name, memories, history, want) in CASES {
        let removal = Removal {
            memories: *memories,
            history: *history,
        };
        assert_eq!(needs_memory(&removal), *want, "{name}");
    }
}
