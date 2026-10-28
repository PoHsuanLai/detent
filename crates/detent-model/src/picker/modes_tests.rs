//! Automatic with its list: what each row says and when it is ready, over a text slot whose
//! models sit in the module's groups. Local's questions are in `local_tests`.

use super::*;
use ds_settings::schema::{
    AgentSetting, ChoiceGroup, ChoicePlace, ChoiceWord, Exposure, Help, KeyKind, KeyPath, KeySpec,
    Label, Page, Section, WordLabels,
};
use std::collections::BTreeMap;

pub(super) const ROW: &str = "ai.model.text.balanced";
const OTHER_TIER: &str = "ai.model.text.fast";
const POOL: &str = "ai.auto.pool.text";

/// The words the slot offers, each with its group.
pub(super) const MODELS: &[(&str, &str)] = &[
    ("local/qwen", "On this computer"),
    ("local/holo", "On this computer"),
    ("ollama/llama3", "On this computer"),
    ("anthropic/claude", "Anthropic"),
];

pub(super) fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

fn groups(models: &[(&str, &str)]) -> BTreeMap<ChoiceWord, ChoiceGroup> {
    models
        .iter()
        .map(|(word, group)| {
            (
                ChoiceWord((*word).to_owned()),
                ChoiceGroup((*group).to_owned()),
            )
        })
        .collect()
}

fn labels(models: &[(&str, &str)]) -> WordLabels {
    let named = models
        .iter()
        .map(|(word, _)| (*word, word.split_once('/').map_or(*word, |(_, name)| name)));
    WordLabels(
        std::iter::once(("auto", "Automatic"))
            .chain(named)
            .map(|(word, label)| (word.to_owned(), label.to_owned()))
            .collect(),
    )
}

/// The place of each model the fixture groups under "On this computer", with no engine named.
fn places(models: &[(&str, &str)]) -> BTreeMap<ChoiceWord, ChoicePlace> {
    models
        .iter()
        .filter(|(_, group)| *group == "On this computer")
        .map(|(word, _)| {
            (
                ChoiceWord((*word).to_owned()),
                ChoicePlace::ThisComputer { engine: None },
            )
        })
        .collect()
}

fn spec(key: &str, label: &str, kind: KeyKind, models: &[(&str, &str)]) -> KeySpec {
    KeySpec {
        ends: None,
        control: None,
        path: path(key),
        kind,
        default: toml::Value::String(String::new()),
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Intelligence,
        section: Section("Models / Text".to_owned()),
        exposure: Exposure::Basic,
        labels: labels(models),
        unavailable: BTreeMap::new(),
        groups: groups(models),
        places: places(models),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

fn words(models: &[(&str, &str)], lead: &[&str]) -> Vec<String> {
    lead.iter()
        .copied()
        .chain(models.iter().map(|(word, _)| *word))
        .map(str::to_owned)
        .collect()
}

fn tier(key: &str, models: &[(&str, &str)]) -> KeySpec {
    let variants = words(models, &["", "auto"]);
    spec(key, "Balanced", KeyKind::Menu { variants }, models)
}

fn pool(models: &[(&str, &str)]) -> KeySpec {
    let variants = words(models, &[]);
    let kind = KeyKind::List(Box::new(KeyKind::Menu { variants }));
    spec(POOL, "Automatic chooses from", kind, models)
}

pub(super) fn described_with(models: &[(&str, &str)], value: &str, held: &[&str]) -> Described {
    Described {
        keys: vec![tier(OTHER_TIER, models), tier(ROW, models), pool(models)],
        values: BTreeMap::from([(path(ROW), value.to_owned())]),
        lists: BTreeMap::from([(
            path(POOL),
            held.iter().map(|word| (*word).to_owned()).collect(),
        )]),
    }
}

pub(super) fn state_of(described: Described) -> PickerState {
    step(PickerState::Looking, PickerInput::Found(Ok(described))).0
}

pub(super) fn ready_with(value: &str, held: &[&str]) -> PickerState {
    state_of(described_with(MODELS, value, held))
}

pub(super) fn ready_of(state: &PickerState) -> &Ready {
    let PickerState::Ready(ready) = state else {
        panic!("the module answered");
    };
    ready
}

pub(super) fn row_of<'a>(state: &'a PickerState, key: &str) -> &'a ModelRow {
    ready_of(state).row(&path(key)).expect("the row")
}

pub(super) fn chose(state: PickerState, word: &str) -> (PickerState, Vec<PickerEffect>) {
    step(
        state,
        PickerInput::Chose {
            path: path(ROW),
            word: word.to_owned(),
        },
    )
}

fn toggled(state: PickerState, word: &str) -> (PickerState, Vec<PickerEffect>) {
    step(
        state,
        PickerInput::PoolToggled {
            path: path(POOL),
            word: word.to_owned(),
        },
    )
}

/// `(case, stored word, the pool, what the row says it still needs)`.
type ReadyCase<'a> = (&'a str, &'a str, &'a [&'a str], Result<(), ChoiceGap>);

#[test]
fn a_choice_is_ready_when_it_can_be_used() {
    let cases: [ReadyCase<'_>; 6] = [
        ("nothing chosen", "", &[], Err(ChoiceGap::Choice)),
        (
            "Automatic with an empty pool",
            "auto",
            &[],
            Err(ChoiceGap::PoolEmpty),
        ),
        (
            "Automatic with a model to choose from",
            "auto",
            &["anthropic/claude"],
            Ok(()),
        ),
        (
            "Automatic whose every model went",
            "auto",
            &["gone/model"],
            Err(ChoiceGap::PoolGone),
        ),
        (
            "Automatic with one model gone and one left",
            "auto",
            &["gone/model", "local/qwen"],
            Ok(()),
        ),
        ("a model", "anthropic/claude", &[], Ok(())),
    ];
    for (case, value, held, want) in cases {
        let state = ready_with(value, held);
        assert_eq!(row_of(&state, ROW).readiness(), want, "{case}");
    }
}

#[test]
fn an_unready_choice_names_what_is_missing_and_nothing_chosen_names_nothing() {
    let empty = ready_with("auto", &[]);
    assert_eq!(
        row_of(&empty, ROW).unready_gap(),
        Some(ChoiceGap::PoolEmpty)
    );
    let gone = ready_with("auto", &["gone/model"]);
    assert_eq!(row_of(&gone, ROW).unready_gap(), Some(ChoiceGap::PoolGone));
    assert_eq!(row_of(&ready_with("", &[]), ROW).unready_gap(), None);
    assert_eq!(
        row_of(&ready_with("auto", &["local/qwen"]), ROW).unready_gap(),
        None
    );
}

/// One pool group as the test reads it: its name and each model with whether it is in.
type ShownGroup<'a> = (Option<&'a str>, Vec<(&'a str, bool)>);

#[test]
fn the_pool_offers_the_pickers_models_without_nothing_and_automatic() {
    let state = ready_with("auto", &["anthropic/claude"]);
    let groups = row_of(&state, ROW).pool.groups();
    let shown: Vec<ShownGroup> = groups
        .iter()
        .map(|group| {
            let entries = group
                .entries
                .iter()
                .map(|entry| (entry.word.as_str(), entry.in_pool))
                .collect();
            let name = match &group.heading {
                PoolHeading::Named(name) => Some(name.as_str()),
                PoolHeading::Unnamed | PoolHeading::Agents | PoolHeading::NoLongerOffered => None,
            };
            (name, entries)
        })
        .collect();
    assert_eq!(
        shown,
        [
            (
                Some("On this computer"),
                vec![
                    ("local/qwen", false),
                    ("local/holo", false),
                    ("ollama/llama3", false)
                ]
            ),
            (Some("Anthropic"), vec![("anthropic/claude", true)]),
        ]
    );
}

#[test]
fn every_tier_of_a_slot_shares_its_pool_and_a_signal_updates_them_all() {
    let state = ready_with("auto", &[]);
    let (state, effects) = step(
        state,
        PickerInput::PoolChanged {
            path: path(POOL),
            words: vec!["local/qwen".to_owned()],
        },
    );
    assert_eq!(effects, []);
    for key in [ROW, OTHER_TIER] {
        assert_eq!(row_of(&state, key).pool.chosen, ["local/qwen"], "{key}");
    }
}

#[test]
fn turning_a_model_on_writes_the_list_and_takes_it_in_when_the_module_has_it() {
    let (state, effects) = toggled(ready_with("auto", &["local/qwen"]), "anthropic/claude");
    assert_eq!(
        effects,
        [PickerEffect::SetPool {
            path: path(POOL),
            words: vec!["local/qwen".to_owned(), "anthropic/claude".to_owned()]
        }]
    );
    assert_eq!(
        row_of(&state, ROW).pool.chosen,
        ["local/qwen"],
        "the list changes when the module has taken it"
    );
    let (_, blocked) = chose(state.clone(), "anthropic/claude");
    assert_eq!(blocked, [], "no pick is taken while the list is written");

    let (state, effects) = step(state, PickerInput::PoolApplied(path(POOL)));
    assert_eq!(effects, []);
    assert_eq!(
        row_of(&state, ROW).pool.chosen,
        ["local/qwen", "anthropic/claude"]
    );
    assert_eq!(ready_of(&state).busy, Busy::Idle);
}

#[test]
fn a_refused_list_is_told_and_stays_as_it_was() {
    let (state, _) = toggled(ready_with("auto", &["local/qwen"]), "local/holo");
    let (state, effects) = step(state, PickerInput::PoolRefused(path(POOL)));
    assert_eq!(effects, [PickerEffect::PoolRefused]);
    assert_eq!(row_of(&state, ROW).pool.chosen, ["local/qwen"]);
    assert_eq!(ready_of(&state).busy, Busy::Idle);
}

#[test]
fn a_model_the_list_holds_and_the_module_dropped_is_listed_apart_and_can_be_removed() {
    let state = ready_with("auto", &["anthropic/claude", "gone/model"]);
    let groups = row_of(&state, ROW).pool.groups();
    let apart = groups.last().expect("a group");
    assert_eq!(apart.heading, PoolHeading::NoLongerOffered);
    let entry = &apart.entries[0];
    assert_eq!(
        (entry.word.as_str(), entry.in_pool, &entry.standing),
        ("gone/model", true, &PoolStanding::Gone)
    );
    assert!(entry.can_toggle(), "it can be taken out");

    let (state, effects) = toggled(state, "gone/model");
    assert_eq!(
        effects,
        [PickerEffect::SetPool {
            path: path(POOL),
            words: vec!["anthropic/claude".to_owned()]
        }]
    );
    let (state, _) = step(state, PickerInput::PoolApplied(path(POOL)));
    assert!(
        row_of(&state, ROW)
            .pool
            .groups()
            .iter()
            .all(|group| group.heading != PoolHeading::NoLongerOffered),
        "nothing is left apart"
    );
    assert_eq!(row_of(&state, ROW).readiness(), Ok(()));
}

#[test]
fn a_model_that_is_blocked_cannot_be_added_but_can_be_taken_out() {
    let mut described = described_with(MODELS, "auto", &["local/holo"]);
    for spec in &mut described.keys {
        if spec.path.0 == POOL {
            spec.unavailable.insert(
                ChoiceWord("local/holo".to_owned()),
                ds_settings::schema::UnavailableReason("Needs a newer version".to_owned()),
            );
            spec.unavailable.insert(
                ChoiceWord("local/qwen".to_owned()),
                ds_settings::schema::UnavailableReason("Needs a newer version".to_owned()),
            );
        }
    }
    let state = state_of(described);
    let (after, effects) = toggled(state.clone(), "local/qwen");
    assert_eq!(
        effects,
        [PickerEffect::Tell("Needs a newer version".to_owned())]
    );
    assert_eq!(after, state, "nothing is written");
    let (_, effects) = toggled(state, "local/holo");
    assert_eq!(
        effects,
        [PickerEffect::SetPool {
            path: path(POOL),
            words: vec![]
        }]
    );
}

#[test]
fn choosing_automatic_writes_the_word_and_never_picks_a_pool_for_the_person() {
    let (state, effects) = chose(ready_with("", &[]), "auto");
    assert_eq!(
        effects,
        [PickerEffect::Set {
            path: path(ROW),
            word: "auto".to_owned()
        }]
    );
    let (state, _) = step(state, PickerInput::Applied(path(ROW)));
    let row = row_of(&state, ROW);
    assert_eq!(row.pool.chosen, Vec::<String>::new());
    assert_eq!(
        row.readiness(),
        Err(ChoiceGap::PoolEmpty),
        "Automatic is not ready until a model is in the list"
    );
}
