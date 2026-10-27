//! Local, with its questions: the menu that lists it, the engine and the model it asks for, and
//! the way to add an engine, over the text slot `modes_tests` builds.

use super::local::LOCAL_ITEM;
use super::modes_tests::{
    MODELS, ROW, chose, described_with, path, ready_of, ready_with, row_of, state_of,
};
use super::*;
use crate::accounts::AddEnd;

#[test]
fn the_menu_lists_local_after_automatic_and_none_of_the_models_this_computer_runs() {
    let state = ready_with("", &[]);
    let lines: Vec<String> = row_of(&state, ROW)
        .menu(Clearing::Offered)
        .into_iter()
        .map(|line| match line {
            MenuLine::Header(name) => format!("# {name}"),
            MenuLine::Pick { word, label, .. } => format!("{word} = {label}"),
        })
        .collect();
    assert_eq!(
        lines,
        [
            " = Off",
            "auto = Automatic",
            "on-this-computer = Local",
            "# Anthropic",
            "anthropic/claude = claude",
        ]
    );
}

#[test]
fn a_menu_that_withholds_clearing_has_no_off() {
    let state = ready_with("", &[]);
    let words: Vec<String> = row_of(&state, ROW)
        .menu(Clearing::Withheld)
        .into_iter()
        .filter_map(|line| match line {
            MenuLine::Pick { word, .. } => Some(word),
            MenuLine::Header(_) => None,
        })
        .collect();
    assert_eq!(words.first().map(String::as_str), Some("auto"));
    assert!(!words.iter().any(String::is_empty));
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
        .map(|engine| (engine.id.as_str(), engine.label.as_str()))
        .collect();
    assert_eq!(engines, [("local", "Local"), ("ollama", "Ollama")]);

    let (state, effects) = step(
        state,
        PickerInput::Engine {
            path: path(ROW),
            engine: "ollama".to_owned(),
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
            engine: "local".to_owned(),
        },
    );
    assert_eq!(same, state, "the engine it already runs changes nothing");
    let (other, _) = step(
        state,
        PickerInput::Engine {
            path: path(ROW),
            engine: "ollama".to_owned(),
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
        ready_of(&waiting).busy,
        Busy::Awaiting(AwaitingAccount {
            path: path(ROW),
            word: None
        })
    );
    let (closed, effects) = step(
        waiting.clone(),
        PickerInput::AccountSheetEnded(AddEnd::Closed),
    );
    assert_eq!(
        (ready_of(&closed).busy.clone(), effects),
        (Busy::Idle, vec![])
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
