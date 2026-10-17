//! The keymap the Shortcuts list sits on: keycap's `KeycapSource` for this computer's layer, the
//! desktop's own bindings and the change watch, and the person's own keymap file, which detent
//! reads as text (to keep the person's comments) and writes.

use crate::error::{FilesError, reason_of};
use crate::values::ConfigHome;
use chordkit::{Keymap, KeymapSource, Watch};
use detent_model::{KeymapText, KeymapTexts, MachineLayer, desktop_keymap};
use ds_helpers::{NewFileMode, ReadCap, ReadError, read_text_capped, update_atomic};
use keycap_client::{KeycapSource, system_bindings};
use std::path::{Path, PathBuf};

/// The most keycap reads of a keymap file; a larger one is not used, so it is not written over.
const MAX_BYTES: ReadCap = ReadCap::kib(32);

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

    /// Writes `text` as the person's keymap through [`update_atomic`] (the file's lock, a
    /// uniquely named temporary file and a rename, the file's mode and a link to it kept), so a
    /// watch sees one whole file and a crash leaves the old one. `base` is the text the edit
    /// started from: when the file holds something else now (another program wrote it since),
    /// nothing is written and the answer is [`FilesError::Changed`], so a stale whole-file write
    /// never puts the other program's change back.
    pub fn save_person(&self, base: &str, text: &str) -> Result<(), FilesError> {
        let person = self.source.person_file();
        if let Some(dir) = person.parent() {
            std::fs::create_dir_all(dir).map_err(|source| FilesError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
        }
        update_atomic(person, MAX_BYTES, NewFileMode::Umask, |current| {
            if current.unwrap_or_default() == base {
                Ok(text.to_owned())
            } else {
                Err(FilesError::Changed {
                    path: person.to_path_buf(),
                })
            }
        })
        .map_err(|error| FilesError::of_update(person, error))
    }
}

fn trouble(why: impl std::fmt::Display) -> KeymapText {
    KeymapText {
        text: String::new(),
        trouble: Some(format!("The keymap file couldn't be used ({why}).")),
    }
}

fn read_text(path: &Path) -> KeymapText {
    match read_text_capped(path, MAX_BYTES) {
        Ok(text) => KeymapText {
            text,
            trouble: None,
        },
        Err(error) if error.is_missing() => KeymapText::default(),
        Err(ReadError::TooBig { cap, .. }) => {
            trouble(format!("it is larger than {} KiB", cap.limit() / 1024))
        }
        Err(ReadError::NotText { .. }) => trouble("it is not plain text"),
        Err(error) => trouble(reason_of(&error)),
    }
}
