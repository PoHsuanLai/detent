use super::*;
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, WordLabels,
};
use std::collections::BTreeMap;

fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

fn menu(key: &str, label: &str) -> KeySpec {
    KeySpec {
        path: path(key),
        kind: KeyKind::Menu {
            variants: ["", "auto", "local/qwen", "local/holo"]
                .map(str::to_owned)
                .to_vec(),
        },
        default: toml::Value::String(String::new()),
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Intelligence,
        section: Section("Models".to_owned()),
        exposure: Exposure::Basic,
        labels: WordLabels(
            [
                ("", "Catalogue default"),
                ("auto", "Automatic"),
                ("local/qwen", "qwen"),
                ("local/holo", "holo"),
            ]
            .map(|(word, label)| (word.to_owned(), label.to_owned()))
            .into(),
        ),
        agent: AgentSetting::HandsOff,
    }
}

fn described() -> Described {
    Described {
        keys: vec![
            menu("ai.model.llm.fast", "Language: Fast"),
            menu("ai.model.llm.balanced", "Language: Balanced"),
            menu("ai.model.llm.best", "Language: Demanding"),
            menu("ai.model.computer_use.balanced", "Computer use: Balanced"),
            menu("ai.local_only", "not a picker row"),
            menu("ai.model.llm.huge", "Language: Huge"),
        ],
        values: BTreeMap::from([(path("ai.model.llm.balanced"), "local/qwen".to_owned())]),
    }
}

fn ready() -> PickerState {
    step(PickerState::Probing, PickerInput::Found(Ok(described()))).0
}

fn value_at(state: &PickerState, key: &str) -> Option<String> {
    let PickerState::Ready(ready) = state else {
        return None;
    };
    ready
        .kinds
        .iter()
        .flat_map(|kind| &kind.rows)
        .find(|row| row.path.0 == key)
        .map(|row| row.value.clone())
}

#[test]
fn rows_group_by_kind_in_the_modules_order_with_the_documented_tier_names() {
    let PickerState::Ready(ready) = ready() else {
        panic!("the module answered");
    };
    let groups: Vec<(String, String, Vec<&str>)> = ready
        .kinds
        .iter()
        .map(|kind| {
            (
                kind.kind.clone(),
                kind.label.clone(),
                kind.rows.iter().map(|row| row.tier).collect(),
            )
        })
        .collect();
    assert_eq!(
        groups,
        [
            (
                "llm".to_owned(),
                "Language".to_owned(),
                vec!["Fast", "Balanced", "Demanding"]
            ),
            (
                "computer_use".to_owned(),
                "Computer use".to_owned(),
                vec!["Balanced"]
            ),
        ],
        "a key that is not ai.model.<kind>.<tier> with a known tier is not a row"
    );
}

#[test]
fn a_row_holds_the_value_the_module_answered_and_the_default_otherwise() {
    let state = ready();
    assert_eq!(
        value_at(&state, "ai.model.llm.balanced").as_deref(),
        Some("local/qwen")
    );
    assert_eq!(value_at(&state, "ai.model.llm.fast").as_deref(), Some(""));
}

#[test]
fn choosing_sets_once_shows_loading_and_keeps_the_old_value_until_it_is_applied() {
    let key = path("ai.model.llm.fast");
    let (state, effects) = step(
        ready(),
        PickerInput::Chose {
            path: key.clone(),
            word: "local/holo".to_owned(),
        },
    );
    assert_eq!(
        effects,
        [PickerEffect::Set {
            path: key.clone(),
            word: "local/holo".to_owned()
        }]
    );
    let PickerState::Ready(applying) = &state else {
        panic!("still ready");
    };
    assert_eq!(
        applying.applying,
        Some(Applying {
            path: key.clone(),
            word: "local/holo".to_owned()
        })
    );
    assert_eq!(value_at(&state, &key.0).as_deref(), Some(""));

    let (done, effects) = step(state, PickerInput::Applied(key.clone()));
    assert_eq!(effects, []);
    assert_eq!(value_at(&done, &key.0).as_deref(), Some("local/holo"));
    let PickerState::Ready(done) = done else {
        panic!("still ready");
    };
    assert_eq!(done.applying, None);
}

#[test]
fn a_refused_set_leaves_the_value_and_tells_the_person() {
    let key = path("ai.model.llm.fast");
    let (state, _) = step(
        ready(),
        PickerInput::Chose {
            path: key.clone(),
            word: "auto".to_owned(),
        },
    );
    let (state, effects) = step(
        state,
        PickerInput::Refused {
            path: key.clone(),
            why: "not permitted".to_owned(),
        },
    );
    assert_eq!(effects, [PickerEffect::Tell("not permitted".to_owned())]);
    assert_eq!(value_at(&state, &key.0).as_deref(), Some(""));
    let PickerState::Ready(state) = state else {
        panic!("still ready");
    };
    assert_eq!(state.applying, None);
}

#[test]
fn a_second_choice_while_one_applies_and_a_choice_of_the_same_word_do_nothing() {
    let (busy, _) = step(
        ready(),
        PickerInput::Chose {
            path: path("ai.model.llm.fast"),
            word: "auto".to_owned(),
        },
    );
    let (same, effects) = step(
        busy.clone(),
        PickerInput::Chose {
            path: path("ai.model.llm.best"),
            word: "auto".to_owned(),
        },
    );
    assert_eq!((same, effects), (busy, vec![]));

    let (idle, effects) = step(
        ready(),
        PickerInput::Chose {
            path: path("ai.model.llm.balanced"),
            word: "local/qwen".to_owned(),
        },
    );
    assert_eq!((idle, effects), (ready(), vec![]));
}

#[test]
fn a_changed_signal_updates_the_row_and_a_lost_module_empties_the_picker() {
    let (state, _) = step(
        ready(),
        PickerInput::Changed {
            path: path("ai.model.computer_use.balanced"),
            word: "local/holo".to_owned(),
        },
    );
    assert_eq!(
        value_at(&state, "ai.model.computer_use.balanced").as_deref(),
        Some("local/holo")
    );
    let (gone, _) = step(state, PickerInput::Found(Err(Absence::NotRunning)));
    assert_eq!(gone, PickerState::Unavailable(Absence::NotRunning));
}

#[test]
fn a_row_says_loading_for_a_model_and_applying_for_the_words_that_name_none() {
    let PickerState::Ready(ready) = ready() else {
        panic!("ready");
    };
    let row = &ready.kinds[0].rows[0];
    for (word, want) in [
        ("local/holo", "Loading holo…"),
        ("auto", "Applying"),
        ("", "Applying"),
    ] {
        assert_eq!(loading_text(row, word), want, "{word:?}");
    }
}
