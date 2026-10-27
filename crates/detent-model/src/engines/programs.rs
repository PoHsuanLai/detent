//! The rows that name the program of an engine (`engines.vllm_python`, `engines.llama_server`),
//! which inferd's settings module lists as text under "Local models": where the person says the
//! program is when the assistant does not find it itself. Empty means it finds it. The module
//! refuses a path it will not run, in plain words, and the row shows them. Pure; the calls are
//! `detent-ui`'s.

use crate::{Absence, Described, Live};
use ds_settings::schema::{Exposure, KeyKind, KeyPath};
use std::collections::BTreeMap;

/// The section of the module whose text rows these are.
pub const PROGRAMS_SECTION: &str = "Local models";

/// One engine's program row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramRow {
    /// The row's key in the module.
    pub path: KeyPath,
    /// The row's name.
    pub label: String,
    /// The module's line about it.
    pub help: String,
    /// The path now in force; empty when the assistant finds the program itself.
    pub value: String,
}

/// The program rows `described` lists, with their values.
pub fn program_rows(described: &Described) -> Vec<ProgramRow> {
    described
        .keys
        .iter()
        .filter(|spec| spec.kind == KeyKind::Text)
        .filter(|spec| spec.exposure == Exposure::Advanced && spec.section.0 == PROGRAMS_SECTION)
        .map(|spec| ProgramRow {
            path: spec.path.clone(),
            label: spec.label.0.clone(),
            help: spec.help.0.clone(),
            value: described
                .values
                .get(&spec.path)
                .cloned()
                .unwrap_or_default(),
        })
        .collect()
}

/// The rows, and the words each refusal left under its row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Programs {
    /// The rows, in the module's order.
    pub rows: Vec<ProgramRow>,
    /// The words the module refused a value with, by row, until the next try or the next change.
    pub refused: BTreeMap<KeyPath, String>,
}

/// Where the programs stand.
pub type ProgramsPane = Live<Programs>;

/// What happens to the rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramsInput {
    /// The module's rows were read, or it could not be.
    Found(Result<Vec<ProgramRow>, Absence>),
    /// The person gave a row a value.
    Set(KeyPath, String),
    /// The module put the value in force.
    Applied(KeyPath, String),
    /// The module refused the value, in these words.
    Refused(KeyPath, String),
    /// The module announced a row's value.
    Changed(KeyPath, String),
}

/// What the rows ask the window to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgramsEffect {
    /// Set the row to the value through the module.
    Set(KeyPath, String),
}

impl Programs {
    fn valued(mut self, path: &KeyPath, value: String) -> Programs {
        for row in self.rows.iter_mut().filter(|row| &row.path == path) {
            row.value.clone_from(&value);
        }
        self
    }

    fn after(mut self, input: ProgramsInput) -> (Programs, Vec<ProgramsEffect>) {
        match input {
            ProgramsInput::Set(path, value) => {
                self.refused.remove(&path);
                (self, vec![ProgramsEffect::Set(path, value)])
            }
            ProgramsInput::Applied(path, value) | ProgramsInput::Changed(path, value) => {
                self.refused.remove(&path);
                (self.valued(&path, value), Vec::new())
            }
            ProgramsInput::Refused(path, words) => {
                self.refused.insert(path, words);
                (self, Vec::new())
            }
            ProgramsInput::Found(_) => (self, Vec::new()),
        }
    }
}

/// The pane after `input`, and what it asks the window to do.
pub fn programs_step(
    pane: ProgramsPane,
    input: ProgramsInput,
) -> (ProgramsPane, Vec<ProgramsEffect>) {
    match input {
        ProgramsInput::Found(read) => (
            pane.found(
                read,
                |rows| Programs {
                    rows,
                    ..Programs::default()
                },
                |old, rows| Programs {
                    rows,
                    refused: old.refused,
                },
            ),
            Vec::new(),
        ),
        other => pane.inner(|programs| programs.after(other)),
    }
}
