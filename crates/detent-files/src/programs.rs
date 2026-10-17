//! The installed apps, read from their desktop entries, so the Privacy page can name a program
//! the way the shell does and draw its icon.

use crate::desktop_paths::{app_id, desktop_files};
use detent_model::{DesktopApp, OriginRule, ProgramBook, SchemaOrigin};
use ds_helpers::{DESKTOP_ENTRY_CAP, DesktopFile, read_text_capped};
use std::borrow::Cow;
use std::collections::BTreeMap;
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

/// The folders the programs of the system are in, where an entry's bare program name is looked
/// for.
const BIN_DIRS: [&str; 6] = [
    "/usr/bin",
    "/usr/local/bin",
    "/bin",
    "/usr/sbin",
    "/usr/local/sbin",
    "/sbin",
];

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
fn app_of(path: &Path, text: &str, origin: SchemaOrigin) -> Option<DesktopApp> {
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
        origin,
    })
}

/// The program folders of `BIN_DIRS` that `rule` finds closed to everyone but an administrator,
/// under the name they are listed by and the one their links lead to.
fn system_bins(rule: &OriginRule) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    for dir in BIN_DIRS.map(PathBuf::from) {
        if rule.judge(&dir) != SchemaOrigin::System {
            continue;
        }
        let real = dir.canonicalize().ok();
        for name in std::iter::once(dir).chain(real) {
            if !found.contains(&name) {
                found.push(name);
            }
        }
    }
    found
}

/// Every application entry under `dirs`, in the order of precedence, each with whether its folder
/// is one `rule` finds only an administrator can change: only such an entry may name a program.
/// A directory that is not there, and a file that cannot be read, is over the entry cap or is not
/// text, are skipped.
pub fn scan_programs(dirs: &[PathBuf], rule: &OriginRule) -> ProgramBook {
    let mut judged: BTreeMap<PathBuf, SchemaOrigin> = BTreeMap::new();
    let apps = desktop_files(dirs)
        .into_iter()
        .filter_map(|path| {
            let text = read_text_capped(&path, DESKTOP_ENTRY_CAP).ok()?;
            let folder = path.parent()?.to_owned();
            let origin = *judged
                .entry(folder)
                .or_insert_with_key(|folder| rule.judge(folder));
            app_of(&path, &text, origin)
        })
        .collect();
    ProgramBook {
        apps,
        system_bins: system_bins(rule),
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
