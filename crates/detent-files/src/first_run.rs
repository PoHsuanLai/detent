//! Whether `detent --first-run` has already run: a one-line file in detent's own state
//! directory, not a settings row (a settings row would be a key some schema must own, and the
//! first run is not a preference).

use ds_helpers::{NewFileMode, write_atomic};
use std::io;
use std::path::PathBuf;

/// The file's name under `<state home>/detent/`.
const FILE: &str = "first-run";

/// What the file holds: only that it exists matters.
const CONTENT: &str = "done\n";

/// `$XDG_STATE_HOME` (or a scratch directory standing in for it).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateHome(pub PathBuf);

/// Whether the first run has happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstRunMark {
    path: PathBuf,
}

/// Where a first run stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstRunStanding {
    /// It has not run: `--first-run` opens the window.
    Pending,
    /// It ran once: `--first-run` exits at once.
    Done,
}

impl FirstRunMark {
    /// The mark under `home`: `<home>/detent/first-run`.
    pub fn under(home: &StateHome) -> FirstRunMark {
        FirstRunMark {
            path: home.0.join("detent").join(FILE),
        }
    }

    /// Whether the first run has happened.
    pub fn standing(&self) -> FirstRunStanding {
        if self.path.is_file() {
            FirstRunStanding::Done
        } else {
            FirstRunStanding::Pending
        }
    }

    /// Records that the first run happened.
    pub fn record(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_atomic(&self.path, CONTENT.as_bytes(), NewFileMode::Umask)
            .map_err(|error| io::Error::new(error.source.kind(), error))
    }
}
