//! A running model's facts: its state, its memory and what can be done to it.

/// What a model's engine is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunStanding {
    /// Not running.
    Stopped,
    /// Starting, not yet answering.
    Starting,
    /// Running and answering.
    Ready,
    /// Being stopped.
    Stopping,
    /// Ended on its own and waiting to be tried again.
    Retrying,
    /// Could not be started, and why in the engine's own words when it said.
    Failed(Option<String>),
}

impl RunStanding {
    /// A stable word for a test or a style to find the state by.
    pub fn slug(&self) -> &'static str {
        match self {
            RunStanding::Stopped => "stopped",
            RunStanding::Starting => "starting",
            RunStanding::Ready => "ready",
            RunStanding::Stopping => "stopping",
            RunStanding::Retrying => "retrying",
            RunStanding::Failed(_) => "failed",
        }
    }

    /// Why it failed, when the engine said.
    pub fn reason(&self) -> Option<&str> {
        match self {
            RunStanding::Failed(reason) => reason.as_deref(),
            _ => None,
        }
    }

    /// What can be done to the model: start one that is not running, stop one that is or is about
    /// to be (which also ends the trying again).
    pub fn action(&self) -> RunAction {
        match self {
            RunStanding::Stopped | RunStanding::Failed(_) => RunAction::Start,
            RunStanding::Starting
            | RunStanding::Ready
            | RunStanding::Stopping
            | RunStanding::Retrying => RunAction::Stop,
        }
    }
}

/// What can be done to a running model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunAction {
    /// Start the model.
    Start,
    /// Stop the model.
    Stop,
}

/// One model the computer runs itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRun {
    /// The model, `account/model`.
    pub model: String,
    /// What its engine is doing.
    pub standing: RunStanding,
    /// The graphics memory it takes, in MiB.
    pub memory_mib: u32,
}

/// What a row draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRowView {
    /// The model, `account/model`: what an action is about.
    pub model: String,
    /// What the model is doing.
    pub standing: RunStanding,
    /// The graphics memory it takes in MiB, while it is ready.
    pub memory_mib: Option<u32>,
    /// What can be done to it.
    pub action: RunAction,
    /// Whether the action waits.
    pub in_flight: bool,
    /// The line under the row: the refusal of the last action, else why the model failed.
    pub note: Option<String>,
}
