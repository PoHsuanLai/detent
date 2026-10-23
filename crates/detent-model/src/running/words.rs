//! A running model's words: its state, its memory and the button it gets.

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
    /// The word the row shows.
    pub fn word(&self) -> &'static str {
        match self {
            RunStanding::Stopped => "Stopped",
            RunStanding::Starting => "Starting\u{2026}",
            RunStanding::Ready => "Ready",
            RunStanding::Stopping => "Stopping\u{2026}",
            RunStanding::Retrying => "Trying again\u{2026}",
            RunStanding::Failed(_) => "Couldn\u{2019}t start",
        }
    }

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

    /// What pressing the row's button does: start a model that is not running, stop one that is
    /// or is about to be (which also ends the trying again).
    pub fn button(&self) -> RunButton {
        match self {
            RunStanding::Stopped | RunStanding::Failed(_) => RunButton::Start,
            RunStanding::Starting
            | RunStanding::Ready
            | RunStanding::Stopping
            | RunStanding::Retrying => RunButton::Stop,
        }
    }
}

/// A row's button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunButton {
    /// Start the model.
    Start,
    /// Stop the model.
    Stop,
}

impl RunButton {
    /// The button's label.
    pub fn label(self) -> &'static str {
        match self {
            RunButton::Start => "Start",
            RunButton::Stop => "Stop",
        }
    }
}

/// Whether a row's button waits for something.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InFlight {
    /// A Start or Stop the person pressed has not ended, or the model is being stopped.
    Yes,
    /// Nothing waits.
    No,
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
    /// The model, `account/model`: what a press is about.
    pub model: String,
    /// The name the row shows.
    pub name: String,
    /// What the model is doing.
    pub standing: RunStanding,
    /// The memory it takes, while it is ready.
    pub memory: Option<String>,
    /// The button the row gets.
    pub button: RunButton,
    /// Whether the button waits.
    pub in_flight: InFlight,
    /// The line under the row: the refusal of the last press, else why the model failed.
    pub note: Option<String>,
}

/// The name of a model `account/model`: the model alone, since the account is the computer's own.
pub fn model_name(model: &str) -> &str {
    model.split_once('/').map_or(model, |(_, name)| name)
}

/// `mib` MiB as the person reads memory: gigabytes rounded to one decimal from 1 GB, else megabytes.
pub fn memory_words(mib: u32) -> String {
    match mib {
        0..=1023 => format!("{mib} MB"),
        _ => {
            let tenths = (u64::from(mib) * 10 + 512) / 1024;
            format!("{}.{} GB", tenths / 10, tenths % 10)
        }
    }
}
