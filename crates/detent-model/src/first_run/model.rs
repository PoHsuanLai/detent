//! What the first-run window shows and what the person has chosen so far.

use crate::accounts::{AccountKey, AccountStatus, AccountsPane};

/// The word `ai.local_only` stores for on.
const ON: &str = "on";
/// The word it stores for off.
const OFF: &str = "off";

/// Whether AI stays on this computer: inferd's `ai.local_only`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalOnly {
    /// No request goes to a cloud account.
    On,
    /// A data class's floor alone decides.
    Off,
}

impl LocalOnly {
    /// The word the row stores.
    pub fn word(self) -> &'static str {
        match self {
            LocalOnly::On => ON,
            LocalOnly::Off => OFF,
        }
    }
}

/// A local runtime the window offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runtime {
    /// Its account.
    pub key: AccountKey,
    /// What it is called ("Ollama").
    pub label: String,
    /// How it stands: a stopped one is listed and cannot be picked.
    pub status: AccountStatus,
}

impl Runtime {
    /// What a person reads for how it stands: a runtime that is up is running.
    pub fn state_label(&self) -> String {
        match self.status {
            AccountStatus::Ok => "Running".to_owned(),
            _ => self.status.label(),
        }
    }

    /// Whether it can be picked now.
    pub fn is_running(&self) -> bool {
        self.status == AccountStatus::Ok
    }

    /// The runtimes `pane` lists, in its order.
    pub fn all_of(pane: &AccountsPane) -> Vec<Runtime> {
        pane.local_runtimes()
            .map(|account| Runtime {
                key: account.key.clone(),
                label: account.label.clone(),
                status: account.status.clone(),
            })
            .collect()
    }
}

/// Where the window is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Waiting for the person.
    Asking,
    /// Writing what was chosen.
    Saving,
    /// Written: the window closes.
    Done,
}

/// The window's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstRun {
    /// Whether AI stays on this computer; on until the person says otherwise.
    pub local_only: LocalOnly,
    /// The local runtimes found, in the order accountd lists them.
    pub runtimes: Vec<Runtime>,
    /// The runtime chosen for the text slot; none leaves it to Automatic.
    pub picked: Option<AccountKey>,
    /// Where it is.
    pub stage: Stage,
}

impl Default for FirstRun {
    fn default() -> Self {
        FirstRun {
            local_only: LocalOnly::On,
            runtimes: Vec::new(),
            picked: None,
            stage: Stage::Asking,
        }
    }
}
