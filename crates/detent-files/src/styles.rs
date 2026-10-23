//! The style files the person wrote: the desktop-wide `quire/style.css` and each app's own
//! `<app>/style.css` under the settings folder. Only the text is read here; what it says about the
//! styles is quire's lint.

use crate::read::text_of;
use crate::values::ConfigHome;
use detent_model::StyleOwner;
use std::path::Path;

/// The file every style sheet is called.
const SHEET: &str = "style.css";

/// The folder of the desktop-wide sheet, which is quire's.
const DESKTOP_FOLDER: &str = "quire";

/// The most app folders looked into: a settings folder holds a few dozen, and a runaway one must
/// not hold the window.
const MOST_FOLDERS: usize = 512;

/// One style file that could be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleSheet {
    /// Whose it is.
    pub owner: StyleOwner,
    /// Its text.
    pub css: String,
}

fn sheet_in(folder: &Path, owner: StyleOwner) -> Option<StyleSheet> {
    let css = text_of(&folder.join(SHEET)).ok().flatten()?;
    Some(StyleSheet { owner, css })
}

/// The style files under `home` that exist and can be read: the desktop-wide one first, then each
/// app folder's in the order of their names. A file that is missing or cannot be read is left out,
/// as is a folder that cannot be listed.
pub fn read_style_sheets(home: &ConfigHome) -> Vec<StyleSheet> {
    let desktop = sheet_in(&home.0.join(DESKTOP_FOLDER), StyleOwner::Desktop);
    let mut folders: Vec<String> = std::fs::read_dir(&home.0)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name != DESKTOP_FOLDER)
        .take(MOST_FOLDERS)
        .collect();
    folders.sort();
    let apps = folders.into_iter().filter_map(|name| {
        let at = home.0.join(&name);
        sheet_in(&at, StyleOwner::App(name))
    });
    desktop.into_iter().chain(apps).collect()
}
