//! Local, one choice of the picker among the others: it asks how models run on this computer
//! (an engine), then which model of that engine, and only the model is written.

use super::choice::{Choice, Gate};
use super::model::ModelRow;
use crate::words::humanize;

/// The group the module puts the models this computer runs in.
pub(super) const ON_THIS_COMPUTER: &str = "On this computer";

/// The menu word of the Local choice. It names no model (a model word has an account before a
/// slash), so it is never written to the module.
pub(crate) const LOCAL_ITEM: &str = "on-this-computer";

/// What the menu calls the Local choice.
pub(super) const LOCAL_LABEL: &str = "Local";

/// The account the models inferd runs itself are served under.
const BUILT_IN_ACCOUNT: &str = "local";

/// What a person reads for the built-in account.
const BUILT_IN_LABEL: &str = "Built in";

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

/// A way to run models on this computer: an account that serves models here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalEngine {
    /// The account, the part of a model word before its slash.
    pub account: String,
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
}

impl LocalView {
    /// The models of the engine chosen; none while no engine is.
    pub fn models(&self) -> &[LocalModel] {
        self.engine
            .as_deref()
            .and_then(|account| self.engines.iter().find(|engine| engine.account == account))
            .map(|engine| engine.models.as_slice())
            .unwrap_or_default()
    }
}

/// The account a model word names.
pub(super) fn account_of(word: &str) -> &str {
    word.split_once('/').map_or(word, |(account, _)| account)
}

/// What a person reads for an engine's account.
fn engine_label(account: &str) -> String {
    if account == BUILT_IN_ACCOUNT {
        BUILT_IN_LABEL.to_owned()
    } else {
        humanize(&account.replace('-', " "))
    }
}

impl ModelRow {
    /// Whether the module says `word` runs on this computer.
    pub(super) fn is_local(&self, word: &str) -> bool {
        self.sections
            .iter()
            .filter(|section| section.group.as_deref() == Some(ON_THIS_COMPUTER))
            .any(|section| section.words.iter().any(|local| local == word))
    }

    /// The ways to run models on this computer for this row, in the order the module lists
    /// their models.
    pub fn engines(&self) -> Vec<LocalEngine> {
        let words = self
            .sections
            .iter()
            .filter(|section| section.group.as_deref() == Some(ON_THIS_COMPUTER))
            .flat_map(|section| section.words.iter());
        words.fold(Vec::new(), |mut engines: Vec<LocalEngine>, word| {
            let account = account_of(word);
            let model = LocalModel {
                word: word.clone(),
                label: self.label_of(word),
                gate: self.gate(word),
            };
            match engines.iter_mut().find(|engine| engine.account == account) {
                Some(engine) => engine.models.push(model),
                None => engines.push(LocalEngine {
                    account: account.to_owned(),
                    label: engine_label(account),
                    models: vec![model],
                }),
            }
            engines
        })
    }

    /// The only engine, when there is exactly one: asking which would be no question.
    pub(super) fn sole_engine(&self) -> Option<String> {
        match self.engines().as_slice() {
            [only] => Some(only.account.clone()),
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
            }),
            Choice::Nothing | Choice::Automatic { .. } | Choice::Model(_) => None,
        }
    }
}
