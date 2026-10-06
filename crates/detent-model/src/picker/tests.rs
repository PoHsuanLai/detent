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
        unavailable: Default::default(),
        groups: Default::default(),
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

fn in_section(mut spec: KeySpec, section: &str, label: &str) -> KeySpec {
    spec.section = Section(section.to_owned());
    spec.label = Label(label.to_owned());
    spec
}

fn ready_of(keys: Vec<KeySpec>) -> Ready {
    let described = Described {
        keys,
        values: BTreeMap::new(),
    };
    let PickerState::Ready(ready) = step(PickerState::Probing, PickerInput::Found(Ok(described))).0
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
        UnavailableReason("Add an account to use".to_owned()),
    );
    let (state, _) = step(
        PickerState::Probing,
        PickerInput::Found(Ok(Described {
            keys: vec![gated],
            values: BTreeMap::new(),
        })),
    );
    let PickerState::Ready(ready) = &state else {
        panic!("ready");
    };
    assert_eq!(
        ready.kinds[0].rows[0].unavailable_reason("local/holo"),
        Some("Add an account to use")
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
        [PickerEffect::Tell("Add an account to use".to_owned())]
    );
    assert_eq!(
        after, state,
        "nothing is applying and the value is unchanged"
    );
}
