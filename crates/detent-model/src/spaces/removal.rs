//! The question before a Space is removed: what its memories and its history become. The choices
//! and the wire form they send are decided here; the window words and draws them.

use super::step::SpaceKey;
use almanac_core::{HistoryFate, MemoryFate, Removal};

/// What is asked of the person about a Space's removal. The first choice of each kind is the one nothing is lost by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovalAsk {
    /// The Space.
    pub space: SpaceKey,
    /// Its name, as the person called it.
    pub name: String,
    /// What becomes of its memories.
    pub memories: MemoryFate,
    /// What becomes of its history.
    pub history: HistoryFate,
}

/// What the person answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalInput {
    /// Picked what becomes of the history.
    History(HistoryFate),
    /// Chose the other fate for the memories: moved instead of deleted, or the reverse.
    FlipMemories,
}

impl RemovalAsk {
    /// The question for `space`: memories move to the apps, history is kept.
    pub fn of(space: SpaceKey, name: String) -> RemovalAsk {
        RemovalAsk {
            space,
            name,
            memories: MemoryFate::MoveToApps,
            history: HistoryFate::Keep,
        }
    }

    /// The question after `input`.
    #[must_use]
    pub fn after(self, input: RemovalInput) -> RemovalAsk {
        match input {
            RemovalInput::History(history) => RemovalAsk { history, ..self },
            RemovalInput::FlipMemories => RemovalAsk {
                memories: match self.memories {
                    MemoryFate::MoveToApps => MemoryFate::Delete,
                    MemoryFate::Delete => MemoryFate::MoveToApps,
                },
                ..self
            },
        }
    }

    /// What the choices send.
    pub fn removal(&self) -> Removal {
        Removal {
            memories: self.memories,
            history: self.history,
        }
    }

    /// Whether Remove destroys something: a delete choice is on.
    pub fn destroys(&self) -> bool {
        deletes(&self.removal())
    }
}

fn deletes(removal: &Removal) -> bool {
    removal.memories == MemoryFate::Delete || removal.history == HistoryFate::Delete
}

/// What a removal does when memory is not running: only memory can delete what it holds, so a
/// delete choice waits for it; one that keeps everything has nothing to tell it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithoutMemory {
    /// Ask the registry to remove the Space anyway.
    Proceed,
    /// Do nothing at all and say why, so no Space is removed with its memories or history left
    /// behind that the person asked to delete.
    Refuse,
}

impl WithoutMemory {
    /// What `removal` does while memory is not running.
    pub fn of(removal: &Removal) -> WithoutMemory {
        if deletes(removal) {
            WithoutMemory::Refuse
        } else {
            WithoutMemory::Proceed
        }
    }
}
