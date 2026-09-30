//! The atomic write.

use crate::error::FilesError;
use crate::values::{ConfigHome, FileValues};
use detent_model::FileSpec;
use std::path::Path;

/// Sets `path` in the nested table at the dotted segments.
fn insert(table: &mut toml::Table, path: &[&str], value: toml::Value) {
    match path {
        [] => {}
        [last] => {
            table.insert((*last).to_owned(), value);
        }
        [first, rest @ ..] => {
            let entry = table
                .entry((*first).to_owned())
                .or_insert_with(|| toml::Value::Table(toml::Table::new()));
            if let toml::Value::Table(inner) = entry {
                insert(inner, rest, value);
            }
        }
    }
}

/// The file's text: one nested table holding every key of `spec` (its value, else its default)
/// and the file's own `version` when it had one. Keys no schema describes are not in it.
fn render(spec: &FileSpec, values: &FileValues) -> Result<String, FilesError> {
    let mut table = toml::Table::new();
    if let Some(version) = &values.version {
        table.insert("version".to_owned(), version.clone());
    }
    for key in &spec.keys {
        let value = values.get(&key.path).unwrap_or(&key.default).clone();
        let path: Vec<&str> = key.path.0.split('.').collect();
        insert(&mut table, &path, value);
    }
    Ok(toml::to_string(&table)?)
}

fn refuse_garbled(path: &Path) -> Result<(), FilesError> {
    match std::fs::read_to_string(path) {
        Ok(text) => text
            .parse::<toml::Table>()
            .map(|_| ())
            .map_err(|error| FilesError::Garbled {
                path: path.to_path_buf(),
                reason: error.to_string(),
            }),
        Err(_) => Ok(()),
    }
}

/// Writes `values` as `spec`'s file: the bytes land in a temporary file beside it and are
/// renamed into place, so a crash cannot leave half a file and a directory watch sees one
/// rename. A file that exists and is not valid TOML is left alone.
pub fn save(home: &ConfigHome, spec: &FileSpec, values: &FileValues) -> Result<(), FilesError> {
    let path = home.path_of(&spec.file);
    refuse_garbled(&path)?;
    let io = |at: &Path| {
        let path = at.to_path_buf();
        move |source| FilesError::Io { path, source }
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io(dir))?;
    }
    let temp = path.with_extension("part");
    std::fs::write(&temp, render(spec, values)?).map_err(io(&temp))?;
    std::fs::rename(&temp, &path).map_err(io(&path))
}
