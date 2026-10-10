//! The keymap the Shortcuts list sits on: keycap's `KeycapSource` for this computer's layer, the
//! desktop's own bindings and the change watch, and the person's own keymap file, which detent
//! reads as text (to keep the person's comments) and writes.

use crate::error::FilesError;
use crate::values::ConfigHome;
use chordkit::{Keymap, KeymapSource, Watch};
use detent_model::{KeymapText, KeymapTexts, MachineLayer, desktop_keymap};
use keycap_client::{KeycapSource, system_bindings};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

/// The most keycap reads of a keymap file; a larger one is not used, so it is not written over.
const MAX_BYTES: u64 = 32 * 1024;

/// Where keycap's daemon would be, under a scratch home: nowhere, so nothing real is asked.
const NO_SOCKET: &str = "keycap/no-daemon.sock";

/// The keymap layers: keycap's source.
#[derive(Debug, Clone)]
pub struct KeymapFiles {
    source: KeycapSource,
}

impl KeymapFiles {
    /// This machine's and person's keymap, from the process environment.
    pub fn standard() -> KeymapFiles {
        KeymapFiles {
            source: KeycapSource::standard(),
        }
    }

    /// The person's keymap under `home` (`<home>/keycap/keymap`), no daemon and no machine
    /// keymap: what a test over scratch directories reads.
    pub fn under(home: &ConfigHome) -> KeymapFiles {
        let dir = home.0.join("keycap");
        KeymapFiles {
            source: KeycapSource::new(home.0.join(NO_SOCKET), PathBuf::new(), dir.join("keymap")),
        }
    }

    /// These files with this computer's keymap read from `dir/keymap`.
    #[must_use]
    pub fn with_machine_dir(self, dir: &Path) -> KeymapFiles {
        let person = self.source.person_file().to_path_buf();
        let socket = person.with_file_name("no-daemon.sock");
        KeymapFiles {
            source: KeycapSource::new(socket, dir.join("keymap"), person),
        }
    }

    /// The standard keymap: the conventions with the desktop's own bindings.
    pub fn base(&self) -> Keymap {
        desktop_keymap(system_bindings())
    }

    /// Both layers as they are now. A person's file that is not there is empty; one that cannot
    /// be used is empty with the reason.
    pub fn read(&self) -> KeymapTexts {
        let machine = self.source.machine();
        KeymapTexts {
            machine: MachineLayer {
                overrides: machine.overrides,
                problems: machine.problems,
            },
            person: read_text(self.source.person_file()),
        }
    }

    /// Calls `on_change` from another thread when either layer changes; keep the `Watch`.
    pub fn watch(&self, on_change: impl Fn() + Send + Sync + 'static) -> Option<Watch> {
        self.source.watch(Box::new(on_change)).ok()
    }

    /// Writes `text` as the person's keymap through a temporary file and a rename, so a watch
    /// sees one whole file and a crash leaves the old one.
    pub fn save_person(&self, text: &str) -> Result<(), FilesError> {
        let person = self.source.person_file();
        let io = |at: &Path| {
            let path = at.to_path_buf();
            move |source| FilesError::Io { path, source }
        };
        if let Some(dir) = person.parent() {
            std::fs::create_dir_all(dir).map_err(io(dir))?;
        }
        let temp = person.with_extension("part");
        std::fs::write(&temp, text).map_err(io(&temp))?;
        std::fs::rename(&temp, person).map_err(io(person))
    }
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
