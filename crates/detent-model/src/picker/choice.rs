//! What a row has chosen, whether that is enough to use, and whether a menu word can be picked.

use super::local::{LOCAL_ITEM, LocalAsk};
use super::model::{AUTO, ModelRow};
use super::pool::AutoPool;

/// What the menu calls the empty word, whatever the module calls it.
pub(super) const NONE_LABEL: &str = "None";

/// The reason the module gives for a hosted model that no account reaches. Picking such a model
/// offers the way to add an account instead of setting it.
pub(super) const NEEDS_ACCOUNT: &str = "Add an account to use";

/// What a row has chosen.
#[derive(Debug, Clone, PartialEq)]
pub enum Choice {
    /// The empty word: no model is chosen, and the row shows no selection.
    Nothing,
    /// The word `auto`: the person asked for Automatic, which chooses among the models in `pool`.
    Automatic {
        /// The models Automatic may use.
        pool: AutoPool,
    },
    /// Local: a model that runs on this computer, as far as the person has answered.
    Local {
        /// The way to run it (an account), once chosen.
        engine: Option<String>,
        /// The model (`account/model`), once chosen.
        model: Option<String>,
    },
    /// A model of any other kind, as `account/model`.
    Model(String),
}

/// What is still needed before a choice can be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceGap {
    /// Nothing is chosen.
    Choice,
    /// Automatic has no model to choose from.
    PoolEmpty,
    /// Every model Automatic was given has gone.
    PoolGone,
    /// Local was picked and nothing on this computer can run a model.
    NoLocalEngine,
    /// Local was picked and no way to run it is chosen.
    LocalEngine,
    /// Local was picked and no model of it is chosen.
    LocalModel,
}

impl ChoiceGap {
    /// The sentence that tells the person what to do next.
    pub fn sentence(self) -> &'static str {
        match self {
            ChoiceGap::Choice => "Choose a model.",
            ChoiceGap::PoolEmpty => "Pick at least one model for Automatic to choose from.",
            ChoiceGap::PoolGone => {
                "The models picked for Automatic are not available now. Pick another."
            }
            ChoiceGap::NoLocalEngine => "Nothing on this computer is set up to run models yet.",
            ChoiceGap::LocalEngine => "Choose how models run on this computer.",
            ChoiceGap::LocalModel => "Choose a model to run on this computer.",
        }
    }
}

/// Whether a choice can be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceReadiness {
    /// It can.
    Ready,
    /// It cannot yet.
    Missing(ChoiceGap),
}

impl Choice {
    /// The menu word to mark as chosen; `None` when nothing is chosen, so no item is marked.
    pub fn selected(&self) -> Option<String> {
        match self {
            Choice::Nothing => None,
            Choice::Automatic { .. } => Some(AUTO.to_owned()),
            Choice::Local { .. } => Some(LOCAL_ITEM.to_owned()),
            Choice::Model(word) => Some(word.clone()),
        }
    }

    /// Whether the choice can be used. A model is as soon as it is chosen; Automatic needs a
    /// model to choose from, and Local the way to run it and the model.
    pub fn readiness(&self) -> ChoiceReadiness {
        match self {
            Choice::Nothing => ChoiceReadiness::Missing(ChoiceGap::Choice),
            Choice::Automatic { pool } => pool.readiness(),
            Choice::Local { engine: None, .. } => ChoiceReadiness::Missing(ChoiceGap::LocalEngine),
            Choice::Local { model: None, .. } => ChoiceReadiness::Missing(ChoiceGap::LocalModel),
            Choice::Local { .. } | Choice::Model(_) => ChoiceReadiness::Ready,
        }
    }
}

/// Whether a menu word can be picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    /// It can.
    Open,
    /// It is a hosted model no account reaches: picking it offers to add an account.
    NeedsAccount,
    /// It cannot be picked now, and this is why.
    Blocked(String),
}

impl Gate {
    /// The line shown under the word when it cannot be picked as it is.
    pub fn note(&self) -> Option<&str> {
        match self {
            Gate::Open => None,
            Gate::NeedsAccount => Some(NEEDS_ACCOUNT),
            Gate::Blocked(reason) => Some(reason),
        }
    }
}

/// The gate a module's reason for a word makes.
pub(super) fn gate_of(reason: Option<&str>) -> Gate {
    match reason {
        None => Gate::Open,
        Some(NEEDS_ACCOUNT) => Gate::NeedsAccount,
        Some(reason) => Gate::Blocked(reason.to_owned()),
    }
}

impl ModelRow {
    /// What the row has chosen now. Local picked and not yet answered counts as chosen: the
    /// stored word changes only when its model is.
    pub fn choice(&self) -> Choice {
        if let LocalAsk::Open { engine } = &self.local {
            return Choice::Local {
                engine: engine.clone().or_else(|| self.sole_engine()),
                model: None,
            };
        }
        match self.value.as_str() {
            "" => Choice::Nothing,
            AUTO => Choice::Automatic {
                pool: self.pool.clone(),
            },
            word if self.is_local(word) => Choice::Local {
                engine: Some(self.engine_of(word).0),
                model: Some(word.to_owned()),
            },
            word => Choice::Model(word.to_owned()),
        }
    }

    /// What the row would have chosen if `word` were stored.
    pub fn choice_when(&self, word: &str) -> Choice {
        ModelRow {
            value: word.to_owned(),
            local: LocalAsk::Closed,
            ..self.clone()
        }
        .choice()
    }

    /// Whether the row's choice can be used.
    pub fn readiness(&self) -> ChoiceReadiness {
        match self.choice() {
            Choice::Local { engine: None, .. } if self.engines().is_empty() => {
                ChoiceReadiness::Missing(ChoiceGap::NoLocalEngine)
            }
            choice => choice.readiness(),
        }
    }

    /// The sentence under the row when what it has chosen cannot be used yet; none when nothing
    /// is chosen (the button already asks) or the choice is ready.
    pub fn unready_note(&self) -> Option<&'static str> {
        match (self.choice(), self.readiness()) {
            (Choice::Nothing, _) | (_, ChoiceReadiness::Ready) => None,
            (_, ChoiceReadiness::Missing(missing)) => Some(missing.sentence()),
        }
    }

    /// The module's note, as the plain line under the row. Automatic's list and Local's missing
    /// engine say it in their own place, so the row does not repeat it.
    pub fn row_note(&self) -> Option<&str> {
        let said_below = match self.choice() {
            Choice::Automatic { .. } => true,
            Choice::Local { .. } => self.engines().is_empty(),
            Choice::Nothing | Choice::Model(_) => false,
        };
        self.note.as_deref().filter(|_| !said_below)
    }

    /// Whether `word` can be picked.
    pub fn gate(&self, word: &str) -> Gate {
        gate_of(self.unavailable_reason(word))
    }
}
