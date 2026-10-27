//! Menus with groups, words that cannot be picked as they are, and a pick that waits for an
//! account to be added, over the rows `tests` builds.

use super::tests::{menu, path, ready_of, value_at};
use super::*;
use crate::accounts::AddEnd;
use ds_settings::schema::{KeyKind, KeySpec};
use std::collections::BTreeMap;

fn grouped_menu() -> KeySpec {
    use ds_settings::schema::{ChoiceGroup, ChoiceWord};
    let mut spec = menu("ai.model.text.fast", "Fast");
    spec.kind = KeyKind::Menu {
        variants: [
            "",
            "auto",
            "local/qwen",
            "anthropic/claude",
            "google/gemini",
            "local/holo",
        ]
        .map(str::to_owned)
        .to_vec(),
    };
    spec.groups = [
        ("local/qwen", "On this computer"),
        ("anthropic/claude", "Anthropic"),
        ("google/gemini", "Google"),
        ("local/holo", "On this computer"),
    ]
    .map(|(word, group)| (ChoiceWord(word.to_owned()), ChoiceGroup(group.to_owned())))
    .into();
    spec
}

#[test]
fn grouped_choices_are_ungrouped_first_then_groups_in_first_appearance_order() {
    let ready = ready_of(vec![
        grouped_menu(),
        menu("ai.model.text.balanced", "Balanced"),
    ]);
    let sections = |row: &ModelRow| -> Vec<(Option<String>, Vec<String>)> {
        row.sections
            .iter()
            .map(|section| (section.group.clone(), section.words.clone()))
            .collect()
    };
    let words = |list: &[&str]| {
        list.iter()
            .map(|word| (*word).to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        sections(&ready.kinds[0].rows[0]),
        [
            (None, words(&["", "auto"])),
            (
                Some("On this computer".to_owned()),
                words(&["local/qwen", "local/holo"])
            ),
            (Some("Anthropic".to_owned()), words(&["anthropic/claude"])),
            (Some("Google".to_owned()), words(&["google/gemini"])),
        ]
    );
    assert_eq!(
        sections(&ready.kinds[0].rows[1]),
        [(None, words(&["", "auto", "local/qwen", "local/holo"]))],
        "a module with no groups lists every choice ungrouped"
    );
}

#[test]
fn an_unavailable_choice_inside_a_group_keeps_its_reason() {
    use ds_settings::schema::{ChoiceWord, UnavailableReason};
    let mut spec = grouped_menu();
    spec.unavailable.insert(
        ChoiceWord("anthropic/claude".to_owned()),
        UnavailableReason("Add an account to use".to_owned()),
    );
    let ready = ready_of(vec![spec]);
    let row = &ready.kinds[0].rows[0];
    let anthropic = row
        .sections
        .iter()
        .find(|section| section.group.as_deref() == Some("Anthropic"))
        .expect("its group");
    assert_eq!(anthropic.words, ["anthropic/claude"]);
    assert_eq!(
        row.unavailable_reason("anthropic/claude"),
        Some("Add an account to use")
    );
}

#[test]
fn an_unavailable_choice_carries_its_reason_and_choosing_it_sets_nothing() {
    use ds_settings::schema::{ChoiceWord, UnavailableReason};
    let mut gated = menu("ai.model.text.fast", "Language: Fast");
    gated.unavailable.insert(
        ChoiceWord("local/holo".to_owned()),
        UnavailableReason("Needs a newer version".to_owned()),
    );
    let (state, _) = step(
        PickerState::Looking,
        PickerInput::Found(Ok(Described {
            keys: vec![gated],
            values: BTreeMap::new(),
            ..Default::default()
        })),
    );
    let PickerState::Ready(ready) = &state else {
        panic!("ready");
    };
    assert_eq!(
        ready.kinds[0].rows[0].unavailable_reason("local/holo"),
        Some("Needs a newer version")
    );
    assert_eq!(
        ready.kinds[0].rows[0].unavailable_reason("local/qwen"),
        None
    );

    let (after, effects) = step(
        state.clone(),
        PickerInput::Chose {
            path: path("ai.model.text.fast"),
            word: "local/holo".to_owned(),
        },
    );
    assert_eq!(
        effects,
        [PickerEffect::Tell("Needs a newer version".to_owned())]
    );
    assert_eq!(
        after, state,
        "nothing is applying and the value is unchanged"
    );
}

#[test]
fn a_stored_word_is_nothing_automatic_or_a_model_and_only_nothing_marks_no_item() {
    // `(stored word, what it stands for, the menu word marked)`.
    let cases: [(&str, Choice, Option<&str>); 3] = [
        ("", Choice::Nothing, None),
        (
            "auto",
            Choice::Automatic {
                pool: Default::default(),
            },
            Some("auto"),
        ),
        (
            "local/qwen",
            Choice::Model("local/qwen".to_owned()),
            Some("local/qwen"),
        ),
    ];
    for (word, choice, marked) in cases {
        let ready = ready_of(vec![menu("ai.model.text.balanced", "Balanced")]);
        let row = &ready.kinds[0].rows[0];
        assert_eq!(row.choice_when(word), choice, "{word:?}");
        assert_eq!(choice.selected().as_deref(), marked, "{word:?}");
    }
}

#[test]
fn the_empty_word_reads_none_whatever_the_module_calls_it_and_claims_no_default() {
    // The module's own labels call the empty word "Catalogue default".
    let ready = ready_of(vec![menu("ai.model.text.balanced", "Balanced")]);
    let row = &ready.kinds[0].rows[0];
    assert_eq!(row.value, "", "nothing the module answered is chosen");
    assert_eq!(row.choice(), Choice::Nothing);
    // `(word, what the menu calls it)`.
    const CASES: &[(&str, &str)] = &[("", "Off"), ("auto", "Automatic"), ("local/qwen", "qwen")];
    for (word, label) in CASES {
        assert_eq!(row.label_of(word), *label, "{word:?}");
    }
}

/// The assistant's row with `local/holo` behind an account the person does not have.
fn gated_text_described() -> Described {
    use ds_settings::schema::{ChoiceWord, UnavailableReason};
    let mut spec = menu("ai.model.text.balanced", "Balanced");
    spec.unavailable.insert(
        ChoiceWord("local/holo".to_owned()),
        UnavailableReason("Add an account to use".to_owned()),
    );
    Described {
        keys: vec![spec],
        values: BTreeMap::from([(path("ai.model.text.balanced"), "auto".to_owned())]),
        lists: BTreeMap::new(),
    }
}

fn gated_text_row() -> PickerState {
    step(
        PickerState::Looking,
        PickerInput::Found(Ok(gated_text_described())),
    )
    .0
}

fn picked_holo() -> (PickerState, Vec<PickerEffect>) {
    step(
        gated_text_row(),
        PickerInput::Chose {
            path: path("ai.model.text.balanced"),
            word: "local/holo".to_owned(),
        },
    )
}

#[test]
fn a_model_that_needs_an_account_offers_the_sheet_and_waits_without_setting() {
    let (state, effects) = picked_holo();
    assert_eq!(effects, [PickerEffect::AddAccount]);
    let PickerState::Ready(ready) = &state else {
        panic!("ready");
    };
    assert_eq!(
        ready.busy,
        Busy::Awaiting(AwaitingAccount {
            path: path("ai.model.text.balanced"),
            word: Some("local/holo".to_owned())
        }),
        "nothing is written yet"
    );
    assert_eq!(ready.adding(), crate::accounts::Adding::SheetOpen);
    assert_eq!(
        value_at(&state, "ai.model.text.balanced").as_deref(),
        Some("auto")
    );

    let (again, effects) = step(
        state.clone(),
        PickerInput::Chose {
            path: path("ai.model.text.balanced"),
            word: "local/qwen".to_owned(),
        },
    );
    assert_eq!((again, effects), (state, vec![]), "one pick at a time");
}

#[test]
fn the_sheet_ending_without_an_account_drops_the_pick_and_tells_why() {
    // `(how the sheet ended, what the person is told)`.
    let cases = [
        (AddEnd::Closed, vec![]),
        (
            AddEnd::Failed,
            vec![PickerEffect::AccountNotAdded(AddEnd::Failed)],
        ),
        (
            AddEnd::Busy,
            vec![PickerEffect::AccountNotAdded(AddEnd::Busy)],
        ),
    ];
    for (end, told) in cases {
        let (waiting, _) = picked_holo();
        let (state, effects) = step(waiting, PickerInput::AccountSheetEnded(end.clone()));
        assert_eq!(effects, told, "{end:?}");
        let PickerState::Ready(ready) = state else {
            panic!("ready");
        };
        assert_eq!(ready.busy, Busy::Idle, "{end:?}");
    }
}

#[test]
fn an_account_that_reaches_the_model_makes_the_pick_and_one_that_does_not_drops_it() {
    let (waiting, _) = picked_holo();
    let (waiting, effects) = step(waiting, PickerInput::AccountSheetEnded(AddEnd::Added));
    assert_eq!(effects, [PickerEffect::ReadAgain]);

    let reached = Described {
        keys: vec![menu("ai.model.text.balanced", "Balanced")],
        values: BTreeMap::from([(path("ai.model.text.balanced"), "auto".to_owned())]),
        ..Default::default()
    };
    let (state, effects) = step(waiting.clone(), PickerInput::Reread(reached));
    assert_eq!(
        effects,
        [PickerEffect::Set {
            path: path("ai.model.text.balanced"),
            word: "local/holo".to_owned()
        }]
    );
    let PickerState::Ready(ready) = state else {
        panic!("ready");
    };
    assert_eq!(
        ready.busy,
        Busy::Applying(Applying {
            path: path("ai.model.text.balanced"),
            word: "local/holo".to_owned()
        })
    );

    let (state, effects) = step(waiting, PickerInput::Reread(gated_text_described()));
    assert_eq!(effects, []);
    let PickerState::Ready(ready) = state else {
        panic!("ready");
    };
    assert_eq!(ready.busy, Busy::Idle);
}
