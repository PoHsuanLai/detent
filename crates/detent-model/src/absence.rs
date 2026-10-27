//! Why a live pane (the model picker, the Accounts pane, the updates page, a system page) is not
//! there. A fact only: each pane's copy says it in its own words.

/// Why a live pane is not there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// The service is there and this computer has none of the hardware.
    NoDevice,
    /// The service did not answer, or answered something that made no sense.
    Failed,
}
