//! Which reads of the module run: one at a time, the latest ask wins.

/// Whether a read of the module is running.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Reads {
    /// None is running.
    #[default]
    Idle,
    /// One is running and nothing has asked since it started.
    InFlight,
    /// One is running and something asked after it started: it is out of date when it ends.
    Stale,
}

impl Reads {
    /// A read is wanted: it starts now when none runs (the answer is `true`), else the running one
    /// reads again after (`false`).
    pub fn ask(self) -> (Self, bool) {
        match self {
            Self::Idle => (Self::InFlight, true),
            Self::InFlight | Self::Stale => (Self::Stale, false),
        }
    }

    /// The running read ended: read once more when something asked meanwhile (`true`).
    pub fn finish(self) -> (Self, bool) {
        match self {
            Self::Stale => (Self::InFlight, true),
            Self::Idle | Self::InFlight => (Self::Idle, false),
        }
    }
}
