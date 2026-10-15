//! Why a live pane (the model picker, the Accounts pane) is not there.

/// Why a live pane is not there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Absence {
    /// The window was started without a bus to look on.
    NoBus,
    /// Nothing serves the module: the daemon is not running.
    NotRunning,
    /// The daemon refused this caller: it does not list Settings in its callers.
    NotPermitted,
    /// Anything else the module or the bus said.
    Failed(String),
}

impl std::fmt::Display for Absence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Absence::NoBus => f.write_str("Settings is not connected to the desktop"),
            Absence::NotRunning => f.write_str("the service is not running"),
            Absence::NotPermitted => f.write_str("Settings was not opened from the desktop"),
            Absence::Failed(_) => f.write_str("the service did not answer"),
        }
    }
}
