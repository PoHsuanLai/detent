//! The keymap files the Shortcuts list sits on: this computer's, read only, and the person's own,
//! which detent writes. Both are keycap's keymap file (`keymap` in keycap's directory), so
//! keycap's own watch applies what is written.

use crate::error::FilesError;
use crate::values::ConfigHome;
use detent_model::{KeymapText, KeymapTexts};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The keymap file's name, in keycap's directory under both homes.
const FILE: &str = "keymap";

/// The directory under the config home that holds the person's keymap.
const DIR: &str = "keycap";

/// The most keycap reads of a keymap file; a larger one is not used, so it is not written over.
const MAX_BYTES: u64 = 32 * 1024;

/// Whether this computer's keymap is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachineKeymap {
    /// None: only the person's is.
    Absent,
    /// The file at this path.
    At(PathBuf),
}

/// The two keymap files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapFiles {
    person: PathBuf,
    machine: MachineKeymap,
}

/// What the files looked like when last checked: a change to either shows as a different stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeymapStamp(Vec<Option<(SystemTime, u64)>>);

impl KeymapFiles {
    /// The person's keymap under `home` (`<home>/keycap/keymap`) and no machine keymap.
    pub fn under(home: &ConfigHome) -> KeymapFiles {
        KeymapFiles {
            person: home.0.join(DIR).join(FILE),
            machine: MachineKeymap::Absent,
        }
    }

    /// These files with this computer's keymap read from `dir/keymap`.
    #[must_use]
    pub fn with_machine_dir(self, dir: &Path) -> KeymapFiles {
        KeymapFiles {
            machine: MachineKeymap::At(dir.join(FILE)),
            ..self
        }
    }

    fn machine_path(&self) -> Option<&Path> {
        match &self.machine {
            MachineKeymap::Absent => None,
            MachineKeymap::At(path) => Some(path),
        }
    }

    /// Both files as they are now. A file that is not there is empty; one that cannot be used
    /// is empty with the reason.
    pub fn read(&self) -> KeymapTexts {
        KeymapTexts {
            machine: self.machine_path().map(read_text).unwrap_or_default(),
            person: read_text(&self.person),
        }
    }

    /// How the files look now.
    pub fn stamp(&self) -> KeymapStamp {
        KeymapStamp(
            self.machine_path()
                .into_iter()
                .chain([self.person.as_path()])
                .map(stamp_of)
                .collect(),
        )
    }

    /// Writes `text` as the person's keymap through a temporary file and a rename, so a watch
    /// sees one whole file and a crash leaves the old one.
    pub fn save_person(&self, text: &str) -> Result<(), FilesError> {
        let io = |at: &Path| {
            let path = at.to_path_buf();
            move |source| FilesError::Io { path, source }
        };
        if let Some(dir) = self.person.parent() {
            std::fs::create_dir_all(dir).map_err(io(dir))?;
        }
        let temp = self.person.with_extension("part");
        std::fs::write(&temp, text).map_err(io(&temp))?;
        std::fs::rename(&temp, &self.person).map_err(io(&self.person))
    }
}

fn stamp_of(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

fn trouble(why: impl std::fmt::Display) -> KeymapText {
    KeymapText {
        text: String::new(),
        trouble: Some(format!("The keymap file couldn't be used ({why}).")),
    }
}

fn read_text(path: &Path) -> KeymapText {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == ErrorKind::NotFound => return KeymapText::default(),
        Err(e) => return trouble(e),
    };
    let mut text = String::new();
    match file.take(MAX_BYTES + 1).read_to_string(&mut text) {
        Ok(n) if u64::try_from(n).is_ok_and(|n| n > MAX_BYTES) => {
            trouble(format!("it is larger than {} KiB", MAX_BYTES / 1024))
        }
        Ok(_) => KeymapText {
            text,
            trouble: None,
        },
        Err(e) if e.kind() == ErrorKind::InvalidData => trouble("it is not plain text"),
        Err(e) => trouble(e),
    }
}
