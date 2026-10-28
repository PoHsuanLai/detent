//! The engines the picker can offer, and how each stands.

/// For whom an engine is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallScope {
    /// Everyone on this computer; the system asks for an administrator's password.
    AllUsers,
    /// This person alone; no password.
    JustMe,
}

/// What can be done about an engine that is not there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineWay {
    /// It can be downloaded, in these ways (the first is the default).
    Download {
        /// The ways offered.
        scopes: Vec<InstallScope>,
        /// How many bytes, when that is known.
        size: Option<u64>,
    },
    /// There is nothing to download for this computer, and the person may point at a copy they
    /// already have.
    Folder,
    /// There is nothing to download for this computer.
    Nothing,
}

/// Where an engine stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineStanding {
    /// It is there and the assistant can use it.
    Ready,
    /// It is there, and an install of it was changed since it was made: it is not used.
    Changed,
    /// It is not there.
    NotInstalled,
    /// It is being downloaded, this many thousandths done.
    Installing(u32),
    /// The last download failed, for this reason.
    Failed(EngineFailure),
}

/// Why an engine did not get installed, or counted. The words are the pane's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineFailure {
    /// Not enough free space for the download and its unpacking.
    NoSpace,
    /// The download could not be finished: no connection, or it broke off.
    Offline,
    /// What arrived is not what was expected, so nothing was installed.
    Corrupt,
    /// The person did not give the administrator's password.
    Declined,
    /// There is no download for this computer.
    Unsupported,
    /// The system's software installer could not install it.
    Package,
    /// The person stopped it.
    Cancelled,
    /// It is already being downloaded.
    Busy,
    /// The assistant is not running, so it cannot count a folder.
    NotRunning,
    /// The chosen folder does not hold the engine.
    NotThere,
    /// Anything else, or a download that could not be started.
    Other,
}

/// One engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineRow {
    /// Its id on the assistant's bus.
    pub id: String,
    /// What the person calls it; the name the picker's "Runs with" menu uses for it.
    pub name: String,
    /// Where it stands.
    pub standing: EngineStanding,
    /// What can be done when it is not there.
    pub way: EngineWay,
}

impl EngineRow {
    /// Whether choosing it asks for something: it is not there for the assistant to use.
    pub fn needs_getting(&self) -> bool {
        !matches!(self.standing, EngineStanding::Ready)
    }
}

/// The row named `name` in `rows`.
pub fn engine_named<'a>(rows: &'a [EngineRow], name: &str) -> Option<&'a EngineRow> {
    rows.iter().find(|row| row.name == name)
}
