//! The atomic write of keys in place.

use crate::error::FilesError;
use crate::read::SETTINGS_CAP;
use crate::values::ConfigHome;
use detent_model::ConfigFile;
use ds_helpers::{NewFileMode, update_atomic};
use ds_settings::schema::KeyPath;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table, Value};

/// Sets `path` in `table` (a real or inline table), making the tables on the way, and leaves
/// every other key exactly as it was.
fn set(table: &mut dyn toml_edit::TableLike, path: &[&str], value: Value) -> Result<(), ()> {
    match path {
        [] => Ok(()),
        [last] => {
            match table.get_mut(last) {
                // The key stays where it is, with its own spacing and comments; only the value
                // is replaced, taking the old value's spacing and trailing comment.
                Some(item) => {
                    let mut value = value;
                    if let Some(old) = item.as_value() {
                        *value.decor_mut() = old.decor().clone();
                    }
                    *item = Item::Value(value);
                }
                None => {
                    table.insert(last, Item::Value(value));
                }
            }
            Ok(())
        }
        [first, rest @ ..] => {
            if table.get(first).is_none() {
                let mut made = Table::new();
                made.set_implicit(true);
                table.insert(first, Item::Table(made));
            }
            let inner = table
                .get_mut(first)
                .and_then(Item::as_table_like_mut)
                .ok_or(())?;
            set(inner, rest, value)
        }
    }
}

fn refuse(path: &Path, reason: String) -> FilesError {
    FilesError::Garbled {
        path: path.to_path_buf(),
        reason,
    }
}

/// The keys to set in one file, each with the value it takes.
pub(crate) type Edits<'a> = [(&'a KeyPath, &'a toml::Value)];

/// `text` (the file's, empty when there is none) with every key of `edits` set; every other byte
/// is the file's own.
fn edited(path: &Path, text: &str, edits: &Edits<'_>) -> Result<String, FilesError> {
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| refuse(path, e.message().to_owned()))?;
    for (key, value) in edits {
        let new: Value = value
            .to_string()
            .parse()
            .map_err(|e: toml_edit::TomlError| refuse(path, e.message().to_owned()))?;
        let parts: Vec<&str> = key.0.split('.').collect();
        set(doc.as_table_mut(), &parts, new).map_err(|()| {
            refuse(
                path,
                format!("{} sits inside a value that is not a table", key.0),
            )
        })?;
    }
    Ok(doc.to_string())
}

/// Writes `value` at `key` in `file`: the file is read, that one key is set and the result lands
/// through [`update_atomic`] (a uniquely named temporary file, flushed, renamed, the directory
/// flushed; the file's mode and a link to it are kept), so a crash cannot leave half a file and
/// a directory watch sees one rename. Every other key, known or not, keeps the text it had
/// (detent is not the program that owns the file, so it never removes anything). A file that
/// exists and cannot be read, or is not valid TOML, is left alone.
pub fn save(
    home: &ConfigHome,
    file: &ConfigFile,
    key: &KeyPath,
    value: &toml::Value,
) -> Result<(), FilesError> {
    save_keys(home, file, &[(key, value)])
}

/// [`save`] for several keys of one file: one read, one write, one rename, so a watch of the
/// file never sees some of the keys changed and not the others.
pub(crate) fn save_keys(
    home: &ConfigHome,
    file: &ConfigFile,
    edits: &Edits<'_>,
) -> Result<(), FilesError> {
    let path = home.path_of(file);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|source| FilesError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
    }
    // The read, the edit and the rename happen under the file's lock, which every program that
    // changes the file through `update_atomic` takes: the window's thread, the intents provider
    // and another program's save of the same file queue instead of putting an old text back.
    update_atomic(&path, SETTINGS_CAP, NewFileMode::Umask, |text| {
        edited(&path, &text.unwrap_or_default(), edits)
    })
    .map_err(|error| FilesError::of_update(&path, error))
}
