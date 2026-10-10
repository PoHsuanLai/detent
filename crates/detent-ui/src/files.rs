//! The values the window shows: every settings file read once, and one key set on an edit.

use crate::env::Env;
use detent_files::{FileValues, FilesError, load, save};
use detent_model::{ConfigFile, FileSpec, KeyEntry};
use dioxus::prelude::{ReadableExt, Signal, WritableExt};
use ds::stack::toast_hub::ToastHub;
use std::collections::BTreeMap;

/// The current values of every file the catalogue reads.
pub type Files = BTreeMap<ConfigFile, FileValues>;

/// A person's change to one key.
#[derive(Debug, Clone, PartialEq)]
pub struct Edit {
    /// The key.
    pub entry: KeyEntry,
    /// Its new value.
    pub value: toml::Value,
}

/// The one place the window reports what a read found and dropped.
fn report(spec: &FileSpec, unknown: &[String], invalid: &[String]) {
    let file = spec.file.relative().display();
    for key in unknown {
        eprintln!("detent: {file}: unknown key `{key}` is not read and is left as it is");
    }
    for key in invalid {
        eprintln!("detent: {file}: {key}");
    }
}

/// Every file of the catalogue, read leniently. What no schema describes, or a control
/// cannot hold, is reported and stays in the file untouched.
pub fn load_all(env: &Env) -> Files {
    env.catalogue
        .files
        .iter()
        .map(|spec| {
            let loaded = load(&env.config_home, spec);
            report(
                spec,
                &loaded
                    .unknown
                    .iter()
                    .map(|k| k.0.clone())
                    .collect::<Vec<_>>(),
                &loaded
                    .invalid
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            );
            (spec.file.clone(), loaded.value)
        })
        .collect()
}

/// The value the window shows for `entry`: the file's, else the key's default.
pub fn value_of(files: &Files, entry: &KeyEntry) -> toml::Value {
    files
        .get(&entry.file)
        .and_then(|values| values.get(&entry.spec.path))
        .cloned()
        .unwrap_or_else(|| entry.spec.default.clone())
}

/// The value the file holds for `entry` when no control can represent it.
pub fn written_value(files: &Files, entry: &KeyEntry) -> Option<toml::Value> {
    files
        .get(&entry.file)
        .and_then(|values| values.as_written(&entry.spec.path))
        .cloned()
}

/// `files` with `edit` applied and written to disk, or why it could not be written.
pub fn apply(env: &Env, files: &Files, edit: &Edit) -> Result<Files, FilesError> {
    let file = &edit.entry.file;
    // An entry always comes from the catalogue, so its file is there; were it not, nothing is
    // written and nothing changes.
    let Some(spec) = env.catalogue.file(file) else {
        return Ok(files.clone());
    };
    let before = files.get(file).cloned().unwrap_or_default();
    let after = before.with(&edit.entry.spec.path, edit.value.clone());
    save(
        &env.config_home,
        &spec.file,
        &edit.entry.spec.path,
        &edit.value,
    )?;
    let mut next = files.clone();
    next.insert(file.clone(), after);
    Ok(next)
}

/// Writes `edit` and shows it in `files`; a refusal is told in a toast and `files` keeps what it
/// held.
pub fn store(env: &Env, mut files: Signal<Files>, toasts: ToastHub, edit: &Edit) {
    let applied = apply(env, &files.peek(), edit);
    match applied {
        Ok(next) => files.set(next),
        Err(error) => toasts.push(error.to_string(), None),
    }
}
