//! The installed apps, read from their desktop entries, so the Privacy page can name a program
//! the way the shell does and draw its icon.

use crate::desktop_paths::{app_id, desktop_files};
use detent_model::{DesktopApp, ProgramBook};
use ds_helpers::{DESKTOP_ENTRY_CAP, DesktopFile, read_text_capped};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// The size an entry's icon is looked up at; the page draws it small and the theme gives the
/// nearest it has.
const ICON_PX: u16 = 64;

/// The first word of an `Exec` or `TryExec` line, with the quotes of a quoted one taken off: the
/// program the line starts.
fn first_word(line: &str) -> Option<String> {
    let line = line.trim_start();
    let word = match line.strip_prefix('"') {
        Some(quoted) => quoted.split('"').next().unwrap_or_default(),
        None => line.split_whitespace().next().unwrap_or_default(),
    };
    (!word.is_empty()).then(|| word.to_owned())
}

/// The file of an entry's `Icon=`: an absolute path as it is, a name through the icon theme.
fn icon_file(icon: &str) -> Option<PathBuf> {
    let path = Path::new(icon);
    if path.is_absolute() {
        return Some(path.to_owned());
    }
    (!icon.is_empty())
        .then(|| freedesktop_icons::lookup(icon).with_size(ICON_PX).find())
        .flatten()
}

/// One entry's text as an app; none for anything but an application, and for a text over the
/// entry cap. The name is the entry's plain `Name`, never a translation.
fn app_of(path: &Path, text: &str) -> Option<DesktopApp> {
    let file = DesktopFile::parse(text, DESKTOP_ENTRY_CAP).ok()?;
    let entry = file.entry()?;
    if entry.string("Type").as_deref() != Some("Application") {
        return None;
    }
    let first = |key: &str| entry.string(key).and_then(|line| first_word(&line));
    Some(DesktopApp {
        id: app_id(path)?,
        name: entry
            .string("Name")
            .map(Cow::into_owned)
            .unwrap_or_default(),
        exec: first("Exec"),
        try_exec: first("TryExec"),
        flatpak: entry.string("X-Flatpak").map(Cow::into_owned),
        icon: entry.string("Icon").and_then(|icon| icon_file(&icon)),
    })
}

/// Every application entry under `dirs`, in the order of precedence. A directory that is not
/// there, and a file that cannot be read, is over the entry cap or is not text, are skipped.
pub fn scan_programs(dirs: &[PathBuf]) -> ProgramBook {
    ProgramBook {
        apps: desktop_files(dirs)
            .into_iter()
            .filter_map(|path| {
                let text = read_text_capped(&path, DESKTOP_ENTRY_CAP).ok()?;
                app_of(&path, &text)
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::first_word;

    #[test]
    fn a_launch_line_starts_the_program_named_first() {
        // `(line, the program)`.
        const CASES: &[(&str, Option<&str>)] = &[
            ("/usr/bin/grim %f", Some("/usr/bin/grim")),
            ("  gimp-2.10 %U", Some("gimp-2.10")),
            ("\"/opt/My App/run\" --flag", Some("/opt/My App/run")),
            ("flatpak run org.gnome.Loupe", Some("flatpak")),
            ("", None),
            ("   ", None),
            ("\"\" x", None),
        ];
        for (line, want) in CASES {
            assert_eq!(first_word(line).as_deref(), *want, "{line:?}");
        }
    }
}
