//! The values the window shows: every settings file read once, and rewritten on an edit.

use crate::env::Env;
use detent_files::{FileValues, FilesError, load, save};
use detent_model::{ConfigFile, FileSpec, KeyEntry};
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
        eprintln!("detent: {file}: unknown key `{key}` is not read and not kept");
    }
    for key in invalid {
        eprintln!("detent: {file}: {key}");
    }
}

/// Every file of the catalogue, read leniently. What no schema describes, or the schema
/// refuses, is reported and left out.
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
    save(&env.config_home, spec, &after)?;
    let mut next = files.clone();
    next.insert(file.clone(), after);
    Ok(next)
}
