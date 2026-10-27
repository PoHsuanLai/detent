//! Local, one choice of the picker among the others: it asks how models run on this computer
//! (an engine), then which model of that engine, and only the model is written.

use super::choice::{Choice, Gate};
use super::model::ModelRow;
use crate::words::humanize;
use ds_settings::schema::ChoicePlace;

/// The menu word of the Local choice. It names no model (a model word has an account before a
/// slash), so it is never written to the module.
pub(crate) const LOCAL_ITEM: &str = "on-this-computer";

/// Whether the person is answering what Local asks, before a model of it is chosen.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LocalAsk {
    /// Local is not being asked about.
    #[default]
    Closed,
    /// Local was picked: `engine` is the way to run models, once chosen.
    Open {
        /// The engine's account.
        engine: Option<String>,
    },
}

/// One model an engine runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalModel {
    /// The stored word, `account/model`.
    pub word: String,
    /// What a person reads for it.
    pub label: String,
    /// Whether it can be picked.
    pub gate: Gate,
}

/// A way to run models on this computer: the engine the module names for them, or, for models
/// it names no engine for, the account that serves them here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEngine {
    /// What picks it: the engine's name, or the account (the part of a model word before its
    /// slash) when the module names no engine.
    pub id: String,
    /// What a person reads for it.
    pub label: String,
    /// The models it runs for the slot, in the module's order.
    pub models: Vec<LocalModel>,
}

/// What Local asks right now: the ways to run, the one chosen, the model chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalView {
    /// Every way to run models here for the slot; none when nothing is set up.
    pub engines: Vec<LocalEngine>,
    /// The engine chosen, once there is one.
    pub engine: Option<String>,
    /// The model chosen, once there is one.
    pub model: Option<String>,
    /// The module's line for the row, said when nothing here can run a model; `None` when it
    /// gave none.
    pub note: Option<String>,
}

impl LocalView {
    /// The models of the engine chosen; none while no engine is.
    pub fn models(&self) -> &[LocalModel] {
        self.engine
            .as_deref()
            .and_then(|id| self.engines.iter().find(|engine| engine.id == id))
            .map(|engine| engine.models.as_slice())
            .unwrap_or_default()
    }
}

/// The account a model word names.
pub(super) fn account_of(word: &str) -> &str {
    word.split_once('/').map_or(word, |(account, _)| account)
}

/// What a person reads for an account that serves models here.
fn account_label(account: &str) -> String {
    humanize(&account.replace('-', " "))
}

impl ModelRow {
    /// Whether the module says `word` runs on this computer.
    pub(super) fn is_local(&self, word: &str) -> bool {
        matches!(
            self.places.get(word),
            Some(ChoicePlace::ThisComputer { .. })
        )
    }

    /// The models the module says run on this computer, in its order.
    fn local_words(&self) -> impl Iterator<Item = &String> {
        self.sections
            .iter()
            .flat_map(|section| section.words.iter())
            .filter(|word| self.is_local(word))
    }

    /// What picks the engine that runs the local model `word`, and what a person reads for it:
    /// the engine's own name, or the account's when the module names no engine.
    pub(super) fn engine_of(&self, word: &str) -> (String, String) {
        if let Some(ChoicePlace::ThisComputer {
            engine: Some(engine),
        }) = self.places.get(word)
        {
            (engine.0.clone(), engine.0.clone())
        } else {
            let account = account_of(word);
            (account.to_owned(), account_label(account))
        }
    }

    /// The ways to run models on this computer for this row, in the order the module lists
    /// their models.
    pub fn engines(&self) -> Vec<LocalEngine> {
        self.local_words()
            .fold(Vec::new(), |mut engines: Vec<LocalEngine>, word| {
                let (id, label) = self.engine_of(word);
                let model = LocalModel {
                    word: word.clone(),
                    label: self.label_of(word),
                    gate: self.gate(word),
                };
                match engines.iter_mut().find(|engine| engine.id == id) {
                    Some(engine) => engine.models.push(model),
                    None => engines.push(LocalEngine {
                        id,
                        label,
                        models: vec![model],
                    }),
                }
                engines
            })
    }

    /// The only engine, when there is exactly one: asking which would be no question.
    pub(super) fn sole_engine(&self) -> Option<String> {
        match self.engines().as_slice() {
            [only] => Some(only.id.clone()),
            _ => None,
        }
    }

    /// What Local asks, when Local is what the row has chosen.
    pub fn local_view(&self) -> Option<LocalView> {
        match self.choice() {
            Choice::Local { engine, model } => Some(LocalView {
                engines: self.engines(),
                engine,
                model,
                note: self.note.clone(),
            }),
            Choice::Nothing | Choice::Automatic { .. } | Choice::Model(_) => None,
        }
    }
}
