//! The sheet before a Space is removed: what its memories and its history become. Words, choices
//! and the wire form it sends are decided here; the window only draws them.

use super::step::SpaceKey;
use almanac_core::{HistoryFate, MemoryFate, Removal};

/// The sheet's state. The first choice of each kind is the one nothing is lost by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovalSheet {
    /// The Space.
    pub space: SpaceKey,
    /// Its name, as the person called it.
    pub name: String,
    /// What becomes of its memories.
    pub memories: MemoryFate,
    /// What becomes of its history.
    pub history: HistoryFate,
}

/// What the person did to the sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetInput {
    /// Picked what becomes of the history.
    History(HistoryFate),
    /// Pressed the link under the body: memories move instead of being deleted, or the reverse.
    FlipMemories,
}

impl RemovalSheet {
    /// The sheet for `space`: memories move to the apps, history is kept.
    pub fn of(space: SpaceKey, name: String) -> RemovalSheet {
        RemovalSheet {
            space,
            name,
            memories: MemoryFate::MoveToApps,
            history: HistoryFate::Keep,
        }
    }

    /// The sheet after `input`.
    #[must_use]
    pub fn after(self, input: SheetInput) -> RemovalSheet {
        match input {
            SheetInput::History(history) => RemovalSheet { history, ..self },
            SheetInput::FlipMemories => RemovalSheet {
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

    /// The title.
    pub fn title(&self) -> String {
        format!("Remove \u{201c}{}\u{201d}?", self.name)
    }

    /// The line under the title.
    pub fn body(&self) -> &'static str {
        match self.memories {
            MemoryFate::MoveToApps => "Its memories move to the apps that saved them.",
            MemoryFate::Delete => "Its memories will be deleted.",
        }
    }

    /// The link button that flips what becomes of the memories.
    pub fn flip_label(&self) -> &'static str {
        match self.memories {
            MemoryFate::MoveToApps => "Delete its memories instead",
            MemoryFate::Delete => "Move its memories instead",
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

/// What the person reads when a delete choice cannot be carried out because memory is not running.
pub const MEMORY_NOT_RUNNING: &str = "Memory isn\u{2019}t running right now, so this Space can\u{2019}t be deleted. Try again in a moment.";

/// The history choices, first the one that keeps it.
pub const HISTORY_CHOICES: [(HistoryFate, &str); 2] = [
    (HistoryFate::Keep, "Keep its history"),
    (HistoryFate::Delete, "Delete its history"),
];

/// The line under the history choices.
pub const HISTORY_NOTE: &str =
    "History is what happened in this Space, like files you saved and messages you sent.";
