use super::*;
use crate::Absence;
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, WordLabels,
};
use std::collections::BTreeMap;

pub(super) fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

pub(super) fn menu(key: &str, label: &str) -> KeySpec {
    KeySpec {
        control: None,
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
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
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
        ],
        values: BTreeMap::from([(path("ai.model.llm.balanced"), "local/qwen".to_owned())]),
        lists: BTreeMap::new(),
    }
}

fn ready() -> PickerState {
    step(PickerState::Looking, PickerInput::Found(Ok(described()))).0
}

pub(super) fn value_at(state: &PickerState, key: &str) -> Option<String> {
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
    let groups: Vec<(Option<&str>, &str, Vec<&str>)> = ready
        .kinds
        .iter()
        .map(|kind| {
            (
                kind.area.as_deref(),
                kind.label.as_str(),
                kind.rows.iter().map(|row| row.label.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(
        groups,
        [
            (
                Some("Models"),
                "Language",
                vec!["Fast", "Balanced", "Demanding"]
            ),
            (Some("Models"), "Computer use", vec!["Balanced"]),
        ],
        "an older module's section is the area and its label's prefix the slot; a key that is \
         not ai.model.<slot>.<tier> is not a row"
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
        applying.busy,
        Busy::Applying(Applying {
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
    assert_eq!(done.busy, Busy::Idle);
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
    assert_eq!(state.busy, Busy::Idle);
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
fn a_row_is_loading_a_model_or_applying_one_of_the_words_that_name_none() {
    let PickerState::Ready(ready) = ready() else {
        panic!("ready");
    };
    let row = &ready.kinds[0].rows[0];
    for (word, want) in [
        ("local/holo", Loading::Model("holo".to_owned())),
        ("auto", Loading::Applying),
        ("", Loading::Applying),
    ] {
        assert_eq!(loading_of(row, word), want, "{word:?}");
    }
}

fn in_section(mut spec: KeySpec, section: &str, label: &str) -> KeySpec {
    spec.section = Section(section.to_owned());
    spec.label = Label(label.to_owned());
    spec
}

pub(super) fn ready_of(keys: Vec<KeySpec>) -> Ready {
    let described = Described {
        keys,
        values: BTreeMap::new(),
        ..Default::default()
    };
    let PickerState::Ready(ready) = step(PickerState::Looking, PickerInput::Found(Ok(described))).0
    else {
        panic!("ready");
    };
    ready
}

#[test]
fn rows_come_from_the_schema_in_describe_order_with_no_slot_table() {
    // Slots a table would never list, in an order no table would use.
    let keys = [
        ("zebra", "Zebra stripes", "fast", "Fast"),
        ("zebra", "Zebra stripes", "balanced", "Balanced"),
        ("alpha", "Alpha", "best", "Demanding"),
    ]
    .map(|(slot, name, tier, label)| {
        in_section(
            menu(&format!("ai.model.{slot}.{tier}"), "x"),
            &format!("Models / {name}"),
            label,
        )
    })
    .to_vec();
    let ready = ready_of(keys);
    let shown: Vec<(Option<&str>, &str, Vec<&str>)> = ready
        .kinds
        .iter()
        .map(|kind| {
            (
                kind.area.as_deref(),
                kind.label.as_str(),
                kind.rows.iter().map(|row| row.label.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (Some("Models"), "Zebra stripes", vec!["Fast", "Balanced"]),
            (Some("Models"), "Alpha", vec!["Demanding"]),
        ]
    );
}

#[test]
fn a_section_with_no_separator_and_a_label_with_no_slot_still_render() {
    let ready = ready_of(vec![
        in_section(menu("ai.model.text.fast", "x"), "Models", "Text: Fast"),
        in_section(menu("ai.model.old.fast", "x"), "Everything", "Fast"),
    ]);
    let shown: Vec<(Option<&str>, &str, &str)> = ready
        .kinds
        .iter()
        .map(|kind| {
            (
                kind.area.as_deref(),
                kind.label.as_str(),
                kind.rows[0].label.as_str(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (Some("Models"), "Text", "Fast"),
            (None, "Everything", "Fast")
        ]
    );
}
