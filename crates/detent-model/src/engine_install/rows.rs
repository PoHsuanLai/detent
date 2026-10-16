//! The engines the picker can offer, and how each stands.

/// For whom an engine is installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallScope {
    /// Everyone on this computer; the system asks for an administrator's password.
    AllUsers,
    /// This person alone; no password.
    JustMe,
}

impl InstallScope {
    /// What the pop-up button says.
    pub fn label(self) -> &'static str {
        match self {
            Self::AllUsers => "All Users",
            Self::JustMe => "Just for Me",
        }
    }
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
    /// The last download failed, for this plain reason.
    Failed(String),
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
