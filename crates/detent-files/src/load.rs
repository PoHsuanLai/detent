//! The lenient read.

use crate::unknown::unknown_keys;
use crate::values::{ConfigHome, FileValues};
use detent_model::{Control, FileSpec};
use ds_settings::schema::KeyPath;
use ds_settings::{InvalidKey, Loaded};
use std::collections::BTreeMap;

/// What `spec`'s file holds: every valid value laid over the keys' defaults. A missing file is
/// the defaults; a bad value costs only its own key (its default is used and it is reported in
/// `invalid`); a key no schema describes is reported in `unknown` and is not part of the
/// result, so the next save drops it. Reading never fails.
pub fn load(home: &ConfigHome, spec: &FileSpec) -> Loaded<FileValues> {
    let path = home.path_of(&spec.file);
    let text = match std::fs::read(&path).map(String::from_utf8) {
        Ok(Ok(text)) => text,
        Ok(Err(error)) => return Loaded::garbled(defaults(spec), error.to_string()),
        Err(_) => return Loaded::clean(defaults(spec)),
    };
    match text.parse::<toml::Table>() {
        Ok(table) => overlay(spec, &table),
        Err(error) => Loaded::garbled(defaults(spec), error.to_string()),
    }
}

fn defaults(spec: &FileSpec) -> FileValues {
    let values: BTreeMap<KeyPath, toml::Value> = spec
        .keys
        .iter()
        .map(|key| (key.path.clone(), key.default.clone()))
        .collect();
    FileValues::from_parts(values, BTreeMap::new())
}

fn lookup<'a>(table: &'a toml::Table, path: &KeyPath) -> Option<&'a toml::Value> {
    let mut parts = path.0.split('.');
    let first = table.get(parts.next()?)?;
    parts.try_fold(first, |value, part| value.as_table()?.get(part))
}

fn overlay(spec: &FileSpec, table: &toml::Table) -> Loaded<FileValues> {
    let mut invalid = Vec::new();
    let mut as_written = BTreeMap::new();
    let values: BTreeMap<KeyPath, toml::Value> = spec
        .keys
        .iter()
        .map(|key| {
            let admitted = lookup(table, &key.path).and_then(|given| {
                let ok = Control::of(key).admit(given);
                if ok.is_none() {
                    invalid.push(InvalidKey {
                        path: key.path.0.clone(),
                        reason: format!(
                            "{given} is not a value this control holds, kept as written"
                        ),
                    });
                    as_written.insert(key.path.clone(), given.clone());
                }
                ok
            });
            (
                key.path.clone(),
                admitted.unwrap_or_else(|| key.default.clone()),
            )
        })
        .collect();
    Loaded {
        value: FileValues::from_parts(values, as_written),
        unknown: unknown_keys(table, &spec.keys, &spec.foreign),
        invalid,
    }
}
