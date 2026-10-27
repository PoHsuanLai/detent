//! The running models' transitions: the list inferd gives, the asks that wait for an answer,
//! and the refusals that stay under their row until the next ask.

use super::model::{InFlight, ModelRun, RunAction, RunRowView, RunStanding};
use crate::{Absence, Live};
use std::collections::BTreeMap;

/// Where the running models stand: looking, not there, or listed.
pub type RunPane = Live<RunBoard>;

/// What the list holds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunBoard {
    /// The models, in the order inferd lists them.
    rows: Vec<ModelRun>,
    /// The asks that have not ended, by model.
    asked: BTreeMap<String, RunAction>,
    /// The words of the last refusal, by model.
    refused: BTreeMap<String, String>,
}

/// What happens to the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunInput {
    /// The list was read, or could not be.
    Found(Result<Vec<ModelRun>, Absence>),
    /// The person asked for the row's action on the model.
    Act(String),
    /// A Start or Stop ended: done, or the words it was refused with.
    Answered(String, Result<(), String>),
}

/// What the list asks the window to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunEffect {
    /// Start the model, and wait until it answers.
    Start(String),
    /// Stop the model.
    Stop(String),
    /// Read the list again.
    Look,
}

impl RunBoard {
    fn of(rows: Vec<ModelRun>) -> RunBoard {
        RunBoard {
            rows,
            ..RunBoard::default()
        }
    }

    /// The same, with the list read again: asks and refusals of a model no longer listed go.
    fn relisted(self, rows: Vec<ModelRun>) -> RunBoard {
        let listed = |model: &String| rows.iter().any(|row| &row.model == model);
        RunBoard {
            asked: self.asked.into_iter().filter(|(m, _)| listed(m)).collect(),
            refused: self
                .refused
                .into_iter()
                .filter(|(m, _)| listed(m))
                .collect(),
            rows,
        }
    }

    fn after(mut self, input: RunInput) -> (RunBoard, Vec<RunEffect>) {
        match input {
            RunInput::Act(model) => {
                let action = self
                    .rows
                    .iter()
                    .find(|row| row.model == model)
                    .map(|row| row.standing.action());
                let Some(action) = action.filter(|_| !self.asked.contains_key(&model)) else {
                    return (self, Vec::new());
                };
                self.asked.insert(model.clone(), action);
                self.refused.remove(&model);
                let effect = match action {
                    RunAction::Start => RunEffect::Start(model),
                    RunAction::Stop => RunEffect::Stop(model),
                };
                (self, vec![effect])
            }
            RunInput::Answered(model, ended) => {
                self.asked.remove(&model);
                if let Err(words) = ended {
                    self.refused.insert(model, words);
                }
                (self, vec![RunEffect::Look])
            }
            RunInput::Found(_) => (self, Vec::new()),
        }
    }

    /// What each row draws.
    pub fn rows(&self) -> Vec<RunRowView> {
        self.rows
            .iter()
            .map(|row| {
                let stopping = row.standing == RunStanding::Stopping;
                let asked = self.asked.contains_key(&row.model);
                RunRowView {
                    model: row.model.clone(),
                    standing: row.standing.clone(),
                    memory_mib: (row.standing == RunStanding::Ready && row.memory_mib > 0)
                        .then_some(row.memory_mib),
                    action: match self.asked.get(&row.model) {
                        Some(asked) => *asked,
                        None => row.standing.action(),
                    },
                    in_flight: if asked || stopping {
                        InFlight::Yes
                    } else {
                        InFlight::No
                    },
                    note: self
                        .refused
                        .get(&row.model)
                        .cloned()
                        .or_else(|| row.standing.reason().map(str::to_owned)),
                }
            })
            .collect()
    }
}

/// The pane after `input`, and what it asks the window to do.
pub fn running_step(pane: RunPane, input: RunInput) -> (RunPane, Vec<RunEffect>) {
    match input {
        RunInput::Found(read) => (
            pane.found(read, RunBoard::of, RunBoard::relisted),
            Vec::new(),
        ),
        other => pane.inner(|board| board.after(other)),
    }
}
