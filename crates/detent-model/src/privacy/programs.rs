//! What a program is called and what it looks like, found from its desktop entry the way the
//! shell finds it for a permission question: the entry whose launch line (`Exec`, or `TryExec`)
//! starts that image, or the entry of the sandboxed app with that id; the file name when no entry
//! says. Pure: the entries are read by whoever builds the book.

use super::pane::{ProgramSubject, SubjectKind};
use std::path::{Path, PathBuf};

/// The most characters of a name that are drawn.
const NAME_MAX_CHARS: usize = 48;

/// What a program that no entry names is called.
const UNNAMED_PROGRAM: &str = "A program";

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
}

/// The installed apps.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProgramBook {
    /// Every entry, in the order of precedence.
    pub apps: Vec<DesktopApp>,
}

/// A program as the page draws it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramLook {
    /// What it is called.
    pub name: String,
    /// Its icon file, when its entry has one.
    pub icon: Option<PathBuf>,
}

fn file_name(path: &str) -> Option<&str> {
    Path::new(path).file_name().and_then(|name| name.to_str())
}

/// Whether `launch` (a word of an entry's `Exec` or `TryExec`) starts the image at `path`: the same
/// path, or a bare name that is the image's file name.
fn starts(launch: &str, path: &str) -> bool {
    launch == path || (!launch.contains('/') && file_name(launch) == file_name(path))
}

impl DesktopApp {
    /// Whether this is the sandboxed app `id`: by its Flatpak id, or its entry's id.
    fn is_sandboxed(&self, id: &str) -> bool {
        self.flatpak.as_deref() == Some(id) || self.id == id
    }

    /// Whether its `Exec` or `TryExec` starts the image at `path`.
    fn runs(&self, path: &str) -> bool {
        [&self.exec, &self.try_exec]
            .into_iter()
            .flatten()
            .any(|launch| starts(launch, path))
    }
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

impl ProgramBook {
    /// The entry of `subject`, when one names it.
    fn app_of(&self, subject: &ProgramSubject) -> Option<&DesktopApp> {
        match subject.kind() {
            SubjectKind::Sandboxed => self.apps.iter().find(|app| app.is_sandboxed(&subject.0)),
            SubjectKind::PrivacyProgram => self.apps.iter().find(|app| app.runs(&subject.0)),
        }
    }

    /// What `subject` is called and looks like: its entry's name and icon, else its file name (or
    /// the app's id), else a plain word.
    pub fn look_of(&self, subject: &ProgramSubject) -> ProgramLook {
        let entry = self.app_of(subject);
        let fallback = match subject.kind() {
            SubjectKind::Sandboxed => Some(subject.0.as_str()).filter(|id| !id.is_empty()),
            SubjectKind::PrivacyProgram => file_name(&subject.0),
        };
        let name = entry
            .map(|app| app.name.as_str())
            .filter(|name| !name.is_empty())
            .or(fallback)
            .unwrap_or(UNNAMED_PROGRAM);
        ProgramLook {
            name: clipped(name, NAME_MAX_CHARS),
            icon: entry.and_then(|app| app.icon.clone()),
        }
    }
}
