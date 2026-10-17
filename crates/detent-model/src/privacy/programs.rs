//! What a program is called and what it looks like. The program is known by its proven identity
//! (the image path, or the sandboxed app's id, that the grant book stores), never by what a
//! desktop entry says about itself: an entry names a program only when it is a system one (in a
//! folder only an administrator can change) whose launch line or app id is that identity. Any
//! other program is shown by its file name and the folder it runs from, so a same-user program
//! cannot dress up as a trusted one by planting an entry. Pure: the entries are read by whoever
//! builds the book.

use super::pane::{ProgramSubject, SubjectKind};
use crate::origin::SchemaOrigin;
use std::path::{Path, PathBuf};

/// The most characters of a name that are drawn.
const NAME_MAX_CHARS: usize = 48;

/// The most characters of a folder that are drawn; the end of a long one is kept, since that is
/// the part that tells programs apart.
const PLACE_MAX_CHARS: usize = 60;

/// What a program that nothing names is called.
const UNNAMED_PROGRAM: &str = "A program";

/// What stands for a character that must not be drawn (a control or a direction override).
const REPLACEMENT: char = '\u{FFFD}';

/// One installed app, from its desktop entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopApp {
    /// The entry's id, without `.desktop`.
    pub id: String,
    /// `Name`.
    pub name: String,
    /// The program `Exec` starts: its first word, with quotes taken off.
    pub exec: Option<String>,
    /// `TryExec`, the program that must exist for the entry to count.
    pub try_exec: Option<String>,
    /// `X-Flatpak`, the Flatpak app id; none for a program on the host.
    pub flatpak: Option<String>,
    /// The entry's icon as a file, when it names one the icon theme has.
    pub icon: Option<PathBuf>,
    /// Whether the entry's folder is one only an administrator can change. Only then does the
    /// entry name anything.
    pub origin: SchemaOrigin,
}

/// The installed apps.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProgramBook {
    /// Every entry, in the order of precedence.
    pub apps: Vec<DesktopApp>,
    /// The folders of programs that only an administrator can change, where an entry's bare
    /// program name (`firefox`) is looked for. A bare name matches an image only in one of them.
    pub system_bins: Vec<PathBuf>,
}

/// A program as the page draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramLook {
    /// What it is called.
    pub name: String,
    /// Its icon file, when a system entry names it.
    pub icon: Option<PathBuf>,
    /// The folder it runs from, when no system entry vouches for its name: the page shows it so
    /// a lookalike is told from the real one.
    pub place: Option<String>,
}

fn file_name(path: &str) -> Option<&str> {
    Path::new(path).file_name().and_then(|name| name.to_str())
}

impl DesktopApp {
    /// Whether this entry may name a program at all.
    fn is_system(&self) -> bool {
        self.origin == SchemaOrigin::System
    }

    /// Whether this is the sandboxed app `id`: by its Flatpak id, or its entry's id.
    fn is_sandboxed(&self, id: &str) -> bool {
        self.flatpak.as_deref() == Some(id) || self.id == id
    }

    /// Whether its `Exec` or `TryExec` starts the image at `path`: the same path, or a bare name
    /// that is the image's file name while the image sits in one of `bins`. An entry of a
    /// sandboxed app starts no host program.
    fn runs(&self, path: &str, bins: &[PathBuf]) -> bool {
        let in_bins = Path::new(path)
            .parent()
            .is_some_and(|folder| bins.iter().any(|bin| bin == folder));
        let starts = |launch: &String| {
            launch == path
                || (in_bins && !launch.contains('/') && file_name(launch) == file_name(path))
        };
        self.flatpak.is_none() && [&self.exec, &self.try_exec].into_iter().flatten().any(starts)
    }
}

/// `text` with every character that could hide or reorder what is drawn (controls, direction
/// overrides and isolates) replaced.
fn plain(text: &str) -> String {
    let hidden = |c: char| {
        c.is_control()
            || matches!(
                c,
                '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
            )
    };
    text.chars()
        .map(|c| if hidden(c) { REPLACEMENT } else { c })
        .collect()
}

/// `text` cut to at most `max` characters, the last an ellipsis when it was cut.
fn clipped(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut kept: String = text.chars().take(max.saturating_sub(1)).collect();
    kept.push('\u{2026}');
    kept
}

/// `text` cut to its last `max` characters, the first an ellipsis when it was cut.
fn clipped_start(text: &str, max: usize) -> String {
    let count = text.chars().count();
    if count <= max {
        return text.to_owned();
    }
    let tail: String = text.chars().skip(count - max.saturating_sub(1)).collect();
    format!("\u{2026}{tail}")
}

impl ProgramBook {
    /// The system entry that names `subject`, when one does.
    fn app_of(&self, subject: &ProgramSubject) -> Option<&DesktopApp> {
        let mut system = self.apps.iter().filter(|app| app.is_system());
        match subject.kind() {
            SubjectKind::Sandboxed => system.find(|app| app.is_sandboxed(&subject.0)),
            SubjectKind::PrivacyProgram => {
                system.find(|app| app.runs(&subject.0, &self.system_bins))
            }
        }
    }

    /// What `subject` is called and looks like: the name and icon of the system entry that names
    /// it, else its file name and folder (or the app's id), else a plain word.
    pub fn look_of(&self, subject: &ProgramSubject) -> ProgramLook {
        let entry = self.app_of(subject);
        let named = entry
            .map(|app| app.name.as_str())
            .filter(|name| !name.is_empty());
        let (fallback, place) = match subject.kind() {
            SubjectKind::Sandboxed => (Some(subject.0.as_str()).filter(|id| !id.is_empty()), None),
            SubjectKind::PrivacyProgram => (file_name(&subject.0), folder_of(&subject.0)),
        };
        let place = place.filter(|_| named.is_none());
        let name = named.or(fallback).unwrap_or(UNNAMED_PROGRAM);
        ProgramLook {
            name: clipped(&plain(name), NAME_MAX_CHARS),
            icon: entry.and_then(|app| app.icon.clone()),
            place: place.map(|folder| clipped_start(&plain(folder), PLACE_MAX_CHARS)),
        }
    }
}

/// The folder an image at `path` is in.
fn folder_of(path: &str) -> Option<&str> {
    Path::new(path)
        .parent()
        .and_then(Path::to_str)
        .filter(|folder| !folder.is_empty())
}
