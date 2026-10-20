//! Where a model runs decides how the picker gathers it: the models this computer runs go to
//! Local, grouped by their engine; the others stay in the module's menu groups. The module's
//! note shows under the row.

use super::local::LOCAL_ITEM;
use super::*;
use ds_settings::schema::{
    AgentSetting, ChoiceGroup, ChoicePlace, ChoiceWord, CompanyName, EngineName, Exposure, Help,
    HostName, KeyKind, KeyPath, KeySpec, Label, Note, Page, Section, WordLabels,
};
use std::collections::BTreeMap;

const ROW: &str = "ai.model.text.fast";
const POOL: &str = "ai.auto.pool.text";
const MISSING: &str = "Running models on this computer needs llama.cpp, which is not installed.";

fn here(engine: Option<&str>) -> ChoicePlace {
    ChoicePlace::ThisComputer {
        engine: engine.map(|name| EngineName(name.to_owned())),
    }
}

/// The words of the slot with where each runs and the group the module puts it in.
fn models() -> Vec<(&'static str, Option<ChoicePlace>, Option<&'static str>)> {
    vec![
        (
            "local/a",
            Some(here(Some("llama.cpp"))),
            Some("On this computer"),
        ),
        (
            "local/b",
            Some(here(Some("llama.cpp"))),
            Some("On this computer"),
        ),
        (
            "local/c",
            Some(here(Some("vLLM"))),
            Some("On this computer"),
        ),
        ("ollama/m", Some(here(None)), Some("On this computer")),
        (
            "home/x",
            Some(ChoicePlace::YourComputer {
                host: HostName("frieren".to_owned()),
            }),
            Some("frieren"),
        ),
        (
            "anthropic/claude",
            Some(ChoicePlace::Cloud {
                company: CompanyName("Anthropic".to_owned()),
            }),
            Some("Anthropic"),
        ),
    ]
}

fn spec(key: &str, kind: KeyKind, note: Option<&str>) -> KeySpec {
    let all = models();
    KeySpec {
        control: None,
        path: KeyPath(key.to_owned()),
        kind,
        default: toml::Value::String(String::new()),
        label: Label("Fast".to_owned()),
        help: Help(String::new()),
        page: Page::Intelligence,
        section: Section("Models / Text".to_owned()),
        exposure: Exposure::Basic,
        labels: WordLabels::default(),
        agent: AgentSetting::HandsOff,
        unavailable: BTreeMap::new(),
        groups: all
            .iter()
            .filter_map(|(word, _, group)| {
                Some((
                    ChoiceWord((*word).to_owned()),
                    ChoiceGroup((*group)?.to_owned()),
                ))
            })
            .collect(),
        places: all
            .iter()
            .filter_map(|(word, place, _)| Some((ChoiceWord((*word).to_owned()), place.clone()?)))
            .collect(),
        note: note.map(|text| Note(text.to_owned())),
    }
}

fn words(lead: &[&str]) -> Vec<String> {
    lead.iter()
        .copied()
        .chain(models().iter().map(|(word, _, _)| *word))
        .map(str::to_owned)
        .collect()
}

fn state(tier: KeySpec, value: &str, note: Option<&str>) -> PickerState {
    let pool = spec(
        POOL,
        KeyKind::List(Box::new(KeyKind::Menu {
            variants: words(&[]),
        })),
        note,
    );
    let described = Described {
        keys: vec![tier, pool],
        values: BTreeMap::from([(KeyPath(ROW.to_owned()), value.to_owned())]),
        lists: BTreeMap::new(),
    };
    step(PickerState::Looking, PickerInput::Found(Ok(described))).0
}

fn row(value: &str, note: Option<&str>) -> ModelRow {
    let tier = spec(
        ROW,
        KeyKind::Menu {
            variants: words(&["", "auto"]),
        },
        note,
    );
    let PickerState::Ready(ready) = state(tier, value, note) else {
        panic!("the module answered");
    };
    ready.kinds[0].rows[0].clone()
}

#[test]
fn local_gathers_the_models_this_computer_runs_by_engine() {
    // (engine id, what a person reads, its models); an engine the module does not name is
    // the account that serves the model.
    let got: Vec<(String, String, Vec<String>)> = row("", None)
        .engines()
        .into_iter()
        .map(|engine| {
            let words = engine.models.into_iter().map(|model| model.word).collect();
            (engine.id, engine.label, words)
        })
        .collect();
    let want = |id: &str, label: &str, words: &[&str]| {
        (
            id.to_owned(),
            label.to_owned(),
            words
                .iter()
                .map(|word| (*word).to_owned())
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(
        got,
        [
            want("llama.cpp", "llama.cpp", &["local/a", "local/b"]),
            want("vLLM", "vLLM", &["local/c"]),
            want("ollama", "Ollama", &["ollama/m"]),
        ]
    );
}

#[test]
fn the_menu_keeps_the_other_places_in_the_modules_groups_and_drops_the_local_group() {
    let lines: Vec<String> = row("", None)
        .menu(Clearing::Offered)
        .into_iter()
        .map(|line| match line {
            MenuLine::Header(name) => format!("# {name}"),
            MenuLine::Pick { word, .. } => word,
        })
        .collect();
    assert_eq!(
        lines,
        [
            "",
            "auto",
            LOCAL_ITEM,
            "# frieren",
            "home/x",
            "# Anthropic",
            "anthropic/claude"
        ]
    );
}

#[test]
fn a_group_called_on_this_computer_means_nothing_without_the_place() {
    let mut tier = spec(
        ROW,
        KeyKind::Menu {
            variants: words(&["", "auto"]),
        },
        None,
    );
    tier.places.remove(&ChoiceWord("ollama/m".to_owned()));
    tier.groups.insert(
        ChoiceWord("anthropic/claude".to_owned()),
        ChoiceGroup("Somewhere else".to_owned()),
    );
    tier.places.insert(
        ChoiceWord("anthropic/claude".to_owned()),
        here(Some("llama.cpp")),
    );
    let PickerState::Ready(ready) = state(tier, "", None) else {
        panic!("the module answered");
    };
    let row = &ready.kinds[0].rows[0];
    assert!(row.is_local("anthropic/claude"));
    assert!(!row.is_local("ollama/m"));
    let engines: Vec<String> = row.engines().into_iter().map(|engine| engine.id).collect();
    assert_eq!(engines, ["llama.cpp", "vLLM"]);
    let menu: Vec<MenuLine> = row.menu(Clearing::Offered);
    let listed = |word: &str| {
        menu.iter()
            .any(|line| matches!(line, MenuLine::Pick { word: own, .. } if own == word))
    };
    assert!(listed("ollama/m") && !listed("anthropic/claude"));
}

#[test]
fn a_stored_model_picks_the_engine_its_place_names() {
    let chosen = row("local/c", None).choice();
    assert_eq!(
        chosen,
        Choice::Local {
            engine: Some("vLLM".to_owned()),
            model: Some("local/c".to_owned())
        }
    );
    assert_eq!(
        row("home/x", None).choice(),
        Choice::Model("home/x".to_owned()),
        "a model on another computer of the person's is not a Local one"
    );
}

#[test]
fn the_note_reaches_the_row_the_pool_and_the_local_question() {
    let with = row("", Some(MISSING));
    assert_eq!(with.note.as_deref(), Some(MISSING));
    assert_eq!(with.pool.note.as_deref(), Some(MISSING));
    let (without, local) = (row("", None), row("local/a", Some(MISSING)));
    assert_eq!((without.note, without.pool.note), (None, None));
    assert_eq!(
        local.local_view().and_then(|view| view.note).as_deref(),
        Some(MISSING)
    );
}

#[test]
fn the_row_says_the_note_unless_the_list_or_the_local_question_does() {
    let ready = |value: &str| row(value, Some(MISSING)).row_note().map(str::to_owned);
    assert_eq!(ready(""), Some(MISSING.to_owned()));
    assert_eq!(ready("local/a"), Some(MISSING.to_owned()));
    assert_eq!(ready("home/x"), Some(MISSING.to_owned()));
    assert_eq!(ready("auto"), None, "Automatic's list says it");
    let mut bare = row("", Some(MISSING));
    bare.sections.clear();
    bare.local = LocalAsk::Open { engine: None };
    assert_eq!(bare.row_note(), None, "Local with nothing to run says it");
}
