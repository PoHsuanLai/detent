//! The program rows: which rows of the module they are, and what a set leaves behind.

use super::programs::{ProgramRow, ProgramsEffect, ProgramsInput, program_rows, programs_step};
use crate::{Described, Live};
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section,
};
use std::collections::BTreeMap;

fn spec(path: &str, kind: KeyKind, exposure: Exposure, section: &str) -> KeySpec {
    KeySpec {
        control: None,
        path: KeyPath(path.to_owned()),
        kind,
        default: toml::Value::String(String::new()),
        label: Label(path.to_owned()),
        help: Help("About it.".to_owned()),
        page: Page::Intelligence,
        section: Section(section.to_owned()),
        exposure,
        labels: Default::default(),
        agent: AgentSetting::HandsOff,
        unavailable: BTreeMap::new(),
        groups: BTreeMap::new(),
        places: BTreeMap::new(),
        note: None,
    }
}

fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

fn row(text: &str, value: &str) -> ProgramRow {
    ProgramRow {
        path: path(text),
        label: text.to_owned(),
        help: "About it.".to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn only_the_advanced_text_rows_of_local_models_are_program_rows() {
    let described = Described {
        keys: vec![
            spec(
                "engines.llama_server",
                KeyKind::Text,
                Exposure::Advanced,
                "Local models",
            ),
            spec("ai.other", KeyKind::Text, Exposure::Advanced, "Models"),
            spec("ai.basic", KeyKind::Text, Exposure::Basic, "Local models"),
            spec(
                "ai.menu",
                KeyKind::Menu {
                    variants: Vec::new(),
                },
                Exposure::Advanced,
                "Local models",
            ),
        ],
        values: BTreeMap::from([(
            path("engines.llama_server"),
            "/usr/bin/llama-server".to_owned(),
        )]),
        lists: BTreeMap::new(),
    };
    let rows = program_rows(&described);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].value, "/usr/bin/llama-server");
}

#[test]
fn a_set_asks_the_module_and_a_refusal_stays_until_the_next_try() {
    let (pane, _) = programs_step(
        Live::Looking,
        ProgramsInput::Found(Ok(vec![row("engines.vllm_python", "")])),
    );
    let (pane, effects) = programs_step(
        pane,
        ProgramsInput::Set(path("engines.vllm_python"), "python".to_owned()),
    );
    assert_eq!(
        effects,
        [ProgramsEffect::Set(
            path("engines.vllm_python"),
            "python".to_owned()
        )]
    );
    let words = "Give the whole path of the program, starting with a slash.";
    let (pane, _) = programs_step(
        pane,
        ProgramsInput::Refused(path("engines.vllm_python"), words.to_owned()),
    );
    let Live::Ready(shown) = &pane else {
        panic!("not listed");
    };
    assert_eq!(
        shown
            .refused
            .get(&path("engines.vllm_python"))
            .map(String::as_str),
        Some(words)
    );
    assert_eq!(shown.rows[0].value, "", "a refused value is not in force");
    let (pane, _) = programs_step(
        pane,
        ProgramsInput::Applied(path("engines.vllm_python"), "/usr/bin/python3".to_owned()),
    );
    let Live::Ready(shown) = &pane else {
        panic!("not listed");
    };
    assert!(shown.refused.is_empty());
    assert_eq!(shown.rows[0].value, "/usr/bin/python3");
}
