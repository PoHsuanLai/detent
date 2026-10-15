//! What a stored word means to the person choosing, and whether the word can be chosen.

use super::model::{AUTO, ModelRow};

/// What the menu calls the empty word, whatever the module calls it.
pub(super) const NONE_LABEL: &str = "None";

/// The reason the module gives for a hosted model that no account reaches. Picking such a model
/// offers the way to add an account instead of setting it.
pub(super) const NEEDS_ACCOUNT: &str = "Add an account to use";

/// What a row's stored word stands for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// The empty word: no model is chosen, and the row shows no selection.
    Nothing,
    /// The word `auto`: the person asked for Automatic.
    Automatic,
    /// A model, as `account/model`.
    Model(String),
}

impl Choice {
    /// The choice `word` stores.
    pub fn of(word: &str) -> Choice {
        match word {
            "" => Choice::Nothing,
            AUTO => Choice::Automatic,
            model => Choice::Model(model.to_owned()),
        }
    }

    /// The menu word to mark as chosen; `None` when nothing is chosen, so no item is marked.
    pub fn selected(&self) -> Option<String> {
        match self {
            Choice::Nothing => None,
            Choice::Automatic => Some(AUTO.to_owned()),
            Choice::Model(word) => Some(word.clone()),
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

impl ModelRow {
    /// What the row has chosen now.
    pub fn choice(&self) -> Choice {
        Choice::of(&self.value)
    }

    /// Whether `word` can be picked.
    pub fn gate(&self, word: &str) -> Gate {
        match self.unavailable_reason(word) {
            None => Gate::Open,
            Some(NEEDS_ACCOUNT) => Gate::NeedsAccount,
            Some(reason) => Gate::Blocked(reason.to_owned()),
        }
    }
}
