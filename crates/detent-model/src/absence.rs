//! Why a live pane (the model picker, the Accounts pane, the updates page) is not there.

/// Why a live pane is not there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Absence {
    /// The window was started without a bus to look on.
    NoBus,
    /// Nothing serves the module: the daemon is not running.
    NotRunning,
    /// The daemon refused this caller: it does not list Settings in its callers.
    NotPermitted,
    /// A program owns the service's name but is not the service, so nothing was read from it or
    /// sent to it.
    Unavailable,
    /// Anything else the module or the bus said.
    Failed(String),
}

impl std::fmt::Display for Absence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Absence::NoBus => f.write_str("Settings is not connected to the desktop"),
            Absence::NotRunning => f.write_str("the service is not running"),
            Absence::NotPermitted => f.write_str("Settings was not opened from the desktop"),
            Absence::Unavailable => f.write_str("the service could not be confirmed"),
            Absence::Failed(_) => f.write_str("the service did not answer"),
        }
    }
}
