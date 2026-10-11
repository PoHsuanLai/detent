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

/// What the one who asked or finished does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadTurn {
    /// Read the module now.
    Read,
    /// Nothing: a running read will read again, or nothing is left to read.
    Rest,
}

impl Reads {
    /// A read is wanted: it starts now when none runs, else the running one reads again after.
    pub fn ask(self) -> (Self, ReadTurn) {
        match self {
            Self::Idle => (Self::InFlight, ReadTurn::Read),
            Self::InFlight | Self::Stale => (Self::Stale, ReadTurn::Rest),
        }
    }

    /// The running read ended: read once more when something asked meanwhile.
    pub fn finish(self) -> (Self, ReadTurn) {
        match self {
            Self::Stale => (Self::InFlight, ReadTurn::Read),
            Self::Idle | Self::InFlight => (Self::Idle, ReadTurn::Rest),
        }
    }
}
