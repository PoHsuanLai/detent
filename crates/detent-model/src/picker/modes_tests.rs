//! Automatic with its list, and Local with its questions: what each row says and when it is
//! ready, over a text slot whose models sit in the module's groups.

use super::local::LOCAL_ITEM;
use super::*;
use crate::accounts::AddEnd;
use ds_settings::schema::{
    AgentSetting, ChoiceGroup, ChoiceWord, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page,
    Section, WordLabels,
};
use std::collections::BTreeMap;

const ROW: &str = "ai.model.text.balanced";
const OTHER_TIER: &str = "ai.model.text.fast";
const POOL: &str = "ai.auto.pool.text";

/// The words the slot offers, each with its group.
const MODELS: &[(&str, &str)] = &[
    ("local/qwen", "On this computer"),
    ("local/holo", "On this computer"),
    ("ollama/llama3", "On this computer"),
    ("anthropic/claude", "Anthropic"),
];

fn path(text: &str) -> KeyPath {
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

fn spec(key: &str, label: &str, kind: KeyKind, models: &[(&str, &str)]) -> KeySpec {
    KeySpec {
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

fn described_with(models: &[(&str, &str)], value: &str, held: &[&str]) -> Described {
    Described {
        keys: vec![tier(OTHER_TIER, models), tier(ROW, models), pool(models)],
        values: BTreeMap::from([(path(ROW), value.to_owned())]),
        lists: BTreeMap::from([(
            path(POOL),
            held.iter().map(|word| (*word).to_owned()).collect(),
        )]),
    }
}

fn state_of(described: Described) -> PickerState {
    step(PickerState::Probing, PickerInput::Found(Ok(described))).0
}

fn ready_with(value: &str, held: &[&str]) -> PickerState {
    state_of(described_with(MODELS, value, held))
}

fn ready_of(state: &PickerState) -> &Ready {
    let PickerState::Ready(ready) = state else {
        panic!("the module answered");
    };
    ready
}

fn row_of<'a>(state: &'a PickerState, key: &str) -> &'a ModelRow {
    ready_of(state).row(&path(key)).expect("the row")
}

fn chose(state: PickerState, word: &str) -> (PickerState, Vec<PickerEffect>) {
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

#[test]
fn a_choice_is_ready_when_it_can_be_used() {
    // `(case, stored word, the pool, what the row says it still needs)`.
    let cases: [(&str, &str, &[&str], ChoiceReadiness); 6] = [
        (
            "nothing chosen",
            "",
            &[],
            ChoiceReadiness::Missing(ChoiceGap::Choice),
        ),
        (
            "Automatic with an empty pool",
            "auto",
            &[],
            ChoiceReadiness::Missing(ChoiceGap::PoolEmpty),
        ),
        (
            "Automatic with a model to choose from",
            "auto",
            &["anthropic/claude"],
            ChoiceReadiness::Ready,
        ),
        (
            "Automatic whose every model went",
            "auto",
            &["gone/model"],
            ChoiceReadiness::Missing(ChoiceGap::PoolGone),
        ),
        (
            "Automatic with one model gone and one left",
            "auto",
            &["gone/model", "local/qwen"],
            ChoiceReadiness::Ready,
        ),
        ("a model", "anthropic/claude", &[], ChoiceReadiness::Ready),
    ];
    for (case, value, held, want) in cases {
        let state = ready_with(value, held);
        assert_eq!(row_of(&state, ROW).readiness(), want, "{case}");
    }
}

#[test]
fn an_unready_choice_says_what_to_do_and_nothing_chosen_says_nothing() {
    let empty = ready_with("auto", &[]);
    assert_eq!(
        row_of(&empty, ROW).unready_note(),
        Some("Pick at least one model for Automatic to choose from.")
    );
    let gone = ready_with("auto", &["gone/model"]);
    assert_eq!(
        row_of(&gone, ROW).unready_note(),
        Some("The models picked for Automatic are not available now. Pick another.")
    );
    assert_eq!(row_of(&ready_with("", &[]), ROW).unready_note(), None);
    assert_eq!(
        row_of(&ready_with("auto", &["local/qwen"]), ROW).unready_note(),
        None
    );
}

#[test]
fn the_pool_offers_the_pickers_models_without_nothing_and_automatic() {
    let state = ready_with("auto", &["anthropic/claude"]);
    let groups = row_of(&state, ROW).pool.groups();
    let shown: Vec<(Option<&str>, Vec<(&str, PoolMembership)>)> = groups
        .iter()
        .map(|group| {
            let entries = group
                .entries
                .iter()
                .map(|entry| (entry.word.as_str(), entry.membership))
                .collect();
            (group.group.as_deref(), entries)
        })
        .collect();
    assert_eq!(
        shown,
        [
            (
                Some("On this computer"),
                vec![
                    ("local/qwen", PoolMembership::Out),
                    ("local/holo", PoolMembership::Out),
                    ("ollama/llama3", PoolMembership::Out)
                ]
            ),
            (
                Some("Anthropic"),
                vec![("anthropic/claude", PoolMembership::In)]
            ),
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
    assert_eq!(ready_of(&state).pooling, None);
}

#[test]
fn a_refused_list_is_said_plainly_and_stays_as_it_was() {
    let (state, _) = toggled(ready_with("auto", &["local/qwen"]), "local/holo");
    let (state, effects) = step(state, PickerInput::PoolRefused(path(POOL)));
    assert_eq!(
        effects,
        [PickerEffect::Tell(
            "Automatic could not use that model, so its list is unchanged.".to_owned()
        )]
    );
    assert_eq!(row_of(&state, ROW).pool.chosen, ["local/qwen"]);
    assert_eq!(ready_of(&state).pooling, None);
}

#[test]
fn a_model_the_list_holds_and_the_module_dropped_is_listed_apart_and_can_be_removed() {
    let state = ready_with("auto", &["anthropic/claude", "gone/model"]);
    let groups = row_of(&state, ROW).pool.groups();
    let apart = groups.last().expect("a group");
    assert_eq!(apart.group.as_deref(), Some("No longer available"));
    let entry = &apart.entries[0];
    assert_eq!(
        (entry.word.as_str(), entry.membership, &entry.standing),
        ("gone/model", PoolMembership::In, &PoolStanding::Gone)
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
            .all(|group| group.group.as_deref() != Some("No longer available")),
        "nothing is left apart"
    );
    assert_eq!(row_of(&state, ROW).readiness(), ChoiceReadiness::Ready);
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
        ChoiceReadiness::Missing(ChoiceGap::PoolEmpty),
        "Automatic is not ready until a model is in the list"
    );
}

#[test]
fn the_menu_lists_local_after_automatic_and_none_of_the_models_this_computer_runs() {
    let state = ready_with("", &[]);
    let lines: Vec<String> = row_of(&state, ROW)
        .menu()
        .into_iter()
        .map(|line| match line {
            MenuLine::Header(name) => format!("# {name}"),
            MenuLine::Pick { word, label, .. } => format!("{word} = {label}"),
        })
        .collect();
    assert_eq!(
        lines,
        [
            " = None",
            "auto = Automatic",
            "on-this-computer = Local",
            "# Anthropic",
            "anthropic/claude = claude",
        ]
    );
}

#[test]
fn local_asks_how_it_runs_then_which_model_and_writes_only_the_model() {
    let state = ready_with("", &[]);
    let (state, effects) = chose(state, LOCAL_ITEM);
    assert_eq!(effects, [], "picking Local writes nothing");
    let row = row_of(&state, ROW);
    assert_eq!(row.value, "");
    assert_eq!(
        row.choice(),
        Choice::Local {
            engine: None,
            model: None
        }
    );
    assert_eq!(
        row.readiness(),
        ChoiceReadiness::Missing(ChoiceGap::LocalEngine)
    );
    let view = row.local_view().expect("Local is open");
    let engines: Vec<(&str, &str)> = view
        .engines
        .iter()
        .map(|engine| (engine.account.as_str(), engine.label.as_str()))
        .collect();
    assert_eq!(engines, [("local", "Built in"), ("ollama", "Ollama")]);

    let (state, effects) = step(
        state,
        PickerInput::Engine {
            path: path(ROW),
            account: "ollama".to_owned(),
        },
    );
    assert_eq!(effects, []);
    let row = row_of(&state, ROW);
    assert_eq!(
        row.readiness(),
        ChoiceReadiness::Missing(ChoiceGap::LocalModel)
    );
    let view = row.local_view().expect("Local is open");
    let models: Vec<&str> = view.models().iter().map(|m| m.word.as_str()).collect();
    assert_eq!(models, ["ollama/llama3"], "only the engine's own models");

    let (state, effects) = chose(state, "ollama/llama3");
    assert_eq!(
        effects,
        [PickerEffect::Set {
            path: path(ROW),
            word: "ollama/llama3".to_owned()
        }]
    );
    let (state, _) = step(state, PickerInput::Applied(path(ROW)));
    let row = row_of(&state, ROW);
    assert_eq!(
        row.choice(),
        Choice::Local {
            engine: Some("ollama".to_owned()),
            model: Some("ollama/llama3".to_owned())
        }
    );
    assert_eq!(row.readiness(), ChoiceReadiness::Ready);
    assert_eq!(row.choice().selected().as_deref(), Some(LOCAL_ITEM));
}

#[test]
fn a_stored_local_model_shows_as_local_and_another_engine_asks_for_its_model() {
    let state = ready_with("local/qwen", &[]);
    assert_eq!(
        row_of(&state, ROW).choice(),
        Choice::Local {
            engine: Some("local".to_owned()),
            model: Some("local/qwen".to_owned())
        }
    );
    let (same, _) = step(
        state.clone(),
        PickerInput::Engine {
            path: path(ROW),
            account: "local".to_owned(),
        },
    );
    assert_eq!(same, state, "the engine it already runs changes nothing");
    let (other, _) = step(
        state,
        PickerInput::Engine {
            path: path(ROW),
            account: "ollama".to_owned(),
        },
    );
    let row = row_of(&other, ROW);
    assert_eq!(
        row.value, "local/qwen",
        "the stored model stays until another is chosen"
    );
    assert_eq!(
        row.readiness(),
        ChoiceReadiness::Missing(ChoiceGap::LocalModel)
    );
}

#[test]
fn picking_anything_but_a_local_model_leaves_local() {
    let (open, _) = chose(ready_with("anthropic/claude", &[]), LOCAL_ITEM);
    assert!(matches!(
        row_of(&open, ROW).choice(),
        Choice::Local { model: None, .. }
    ));
    let (back, effects) = chose(open, "anthropic/claude");
    assert_eq!(effects, [], "the model it already has is not written again");
    assert_eq!(
        row_of(&back, ROW).choice(),
        Choice::Model("anthropic/claude".to_owned())
    );
}

#[test]
fn with_one_engine_local_does_not_ask_which() {
    let one = [("local/qwen", "On this computer")];
    let (state, _) = chose(state_of(described_with(&one, "", &[])), LOCAL_ITEM);
    assert_eq!(
        row_of(&state, ROW).choice(),
        Choice::Local {
            engine: Some("local".to_owned()),
            model: None
        }
    );
}

#[test]
fn local_with_nothing_set_up_says_so_and_the_way_to_add_one_waits_for_the_account() {
    let none = [("anthropic/claude", "Anthropic")];
    let (state, _) = chose(state_of(described_with(&none, "", &[])), LOCAL_ITEM);
    let row = row_of(&state, ROW);
    assert_eq!(
        row.readiness(),
        ChoiceReadiness::Missing(ChoiceGap::NoLocalEngine)
    );
    assert_eq!(row.engines(), []);

    let (waiting, effects) = step(state, PickerInput::SetUpEngine(path(ROW)));
    assert_eq!(effects, [PickerEffect::AddAccount]);
    assert_eq!(
        ready_of(&waiting).awaiting,
        Some(AwaitingAccount {
            path: path(ROW),
            word: None
        })
    );
    let (closed, effects) = step(
        waiting.clone(),
        PickerInput::AccountSheetEnded(AddEnd::Closed),
    );
    assert_eq!(
        (ready_of(&closed).awaiting.clone(), effects),
        (None, vec![])
    );
    assert_eq!(row_of(&closed, ROW).value, "", "nothing was written");

    let (added, effects) = step(waiting, PickerInput::AccountSheetEnded(AddEnd::Added));
    assert_eq!(effects, [PickerEffect::ReadAgain]);
    let (again, effects) = step(added, PickerInput::Reread(described_with(MODELS, "", &[])));
    assert_eq!(effects, [], "an engine added sets nothing");
    let row = row_of(&again, ROW);
    assert_eq!(row.engines().len(), 2, "the engines are there now");
    assert!(
        matches!(row.choice(), Choice::Local { model: None, .. }),
        "Local is open again to choose its model"
    );
}
