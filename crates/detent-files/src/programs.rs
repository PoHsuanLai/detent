//! The installed apps, read from their desktop entries, so the Privacy page can name a program
//! the way the shell does and draw its icon.

use detent_model::{DesktopApp, ProgramBook};
use freedesktop_desktop_entry::{DesktopEntry, Iter};
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

/// One entry's text as an app; none for anything but an application or an entry with no name
/// and no launch line to go by.
fn app_of(path: &Path, text: &str) -> Option<DesktopApp> {
    let entry = DesktopEntry::from_str(path, text, None::<&[&str]>).ok()?;
    if entry.type_() != Some("Application") {
        return None;
    }
    let none: &[&str] = &[];
    Some(DesktopApp {
        id: entry.id().to_owned(),
        name: entry
            .name(none)
            .map(|name| name.into_owned())
            .unwrap_or_default(),
        exec: entry.exec().and_then(first_word),
        try_exec: entry.try_exec().and_then(first_word),
        flatpak: entry.flatpak().map(str::to_owned),
        icon: entry.icon().and_then(icon_file),
    })
}

/// Every application entry under `dirs`, in the order of precedence. A directory that is not
/// there, and a file that cannot be read or parsed, are skipped.
pub fn scan_programs(dirs: &[PathBuf]) -> ProgramBook {
    ProgramBook {
        apps: Iter::new(dirs.iter().cloned())
            .filter_map(|path| {
                let text = std::fs::read_to_string(&path).ok()?;
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
