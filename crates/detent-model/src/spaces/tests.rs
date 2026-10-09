//! The removal sheet's words and wire, and the list's transitions.

use super::removal::{HISTORY_CHOICES, SheetInput};
use super::{
    RemovalSheet, SpaceKey, SpaceRow, SpacesEffect, SpacesInput, SpacesState, spaces_step,
};
use almanac_core::{HistoryFate, MemoryFate, Removal};

fn trip() -> RemovalSheet {
    RemovalSheet::of(SpaceKey("space-1".to_owned()), "Trip".to_owned())
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
fn the_sheet_opens_on_the_choices_nothing_is_lost_by() {
    let sheet = trip();
    assert_eq!(sheet.title(), "Remove \u{201c}Trip\u{201d}?");
    assert_eq!(
        sheet.body(),
        "Its memories move to the apps that saved them."
    );
    assert_eq!(sheet.flip_label(), "Delete its memories instead");
    assert_eq!(
        HISTORY_CHOICES[0].0,
        HistoryFate::Keep,
        "the first is selected"
    );
    assert!(!sheet.destroys());
    assert_eq!(sheet.removal(), Removal::KEEP_ALL);
}

#[test]
fn the_link_flips_the_body_and_its_own_label_and_back() {
    let flipped = trip().after(SheetInput::FlipMemories);
    assert_eq!(flipped.body(), "Its memories will be deleted.");
    assert_eq!(flipped.flip_label(), "Move its memories instead");
    assert!(flipped.destroys());
    assert_eq!(flipped.after(SheetInput::FlipMemories), trip());
}

#[test]
fn each_delete_choice_alone_makes_remove_destructive() {
    let history = trip().after(SheetInput::History(HistoryFate::Delete));
    assert!(history.destroys());
    assert_eq!(history.memories, MemoryFate::MoveToApps);
    let memories = trip().after(SheetInput::FlipMemories);
    assert!(memories.destroys());
    assert_eq!(memories.history, HistoryFate::Keep);
}

#[test]
fn the_choices_send_the_json_memory_reads() {
    let both = trip()
        .after(SheetInput::FlipMemories)
        .after(SheetInput::History(HistoryFate::Delete));
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
fn asking_opens_the_sheet_and_cancel_sends_nothing() {
    let SpacesState::Ready(ready) = asked() else {
        panic!("the list is there");
    };
    assert_eq!(ready.sheet, Some(trip()));
    let (after, effects) = spaces_step(SpacesState::Ready(ready), SpacesInput::Cancel);
    assert!(effects.is_empty(), "Cancel sends nothing");
    let SpacesState::Ready(after) = after else {
        panic!("still the list");
    };
    assert_eq!(after.sheet, None);
}

#[test]
fn confirming_sends_the_chosen_removal_for_that_space_once() {
    let changed = spaces_step(
        asked(),
        SpacesInput::Sheet(SheetInput::History(HistoryFate::Delete)),
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
        "the sheet is gone, so a second press sends nothing"
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
    assert_eq!(after.sheet, None);
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
    assert_eq!(after.removing, None);
}
