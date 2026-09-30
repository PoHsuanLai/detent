//! The atomic, single-key write.

use crate::error::FilesError;
use crate::values::ConfigHome;
use detent_model::ConfigFile;
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

/// The file's text with `path` set to `value`; every other byte is the file's own.
fn edited(path: &Path, key: &KeyPath, value: &toml::Value) -> Result<String, FilesError> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| refuse(path, e.to_string()))?;
    let new: Value = value
        .to_string()
        .parse()
        .map_err(|e: toml_edit::TomlError| refuse(path, e.to_string()))?;
    let parts: Vec<&str> = key.0.split('.').collect();
    set(doc.as_table_mut(), &parts, new).map_err(|()| {
        refuse(
            path,
            format!("{} sits inside a value that is not a table", key.0),
        )
    })?;
    Ok(doc.to_string())
}

/// Writes `value` at `key` in `file`: the file is read, that one key is set and the result lands
/// through a temporary file and a rename, so a crash cannot leave half a file and a directory
/// watch sees one rename. Every other key, known or not, keeps the text it had (detent is not
/// the program that owns the file, so it never removes anything). A file that exists and is not
/// valid TOML is left alone.
pub fn save(
    home: &ConfigHome,
    file: &ConfigFile,
    key: &KeyPath,
    value: &toml::Value,
) -> Result<(), FilesError> {
    let path = home.path_of(file);
    let text = edited(&path, key, value)?;
    let io = |at: &Path| {
        let path = at.to_path_buf();
        move |source| FilesError::Io { path, source }
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io(dir))?;
    }
    let temp = path.with_extension("part");
    std::fs::write(&temp, text).map_err(io(&temp))?;
    std::fs::rename(&temp, &path).map_err(io(&path))
}
