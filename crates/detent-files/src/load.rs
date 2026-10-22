//! The lenient read.

use crate::error::reason_of;
use crate::read::text_of;
use crate::unknown::unknown_keys;
use crate::values::{ConfigHome, FileValues};
use detent_model::{Control, FileSpec};
use ds_settings::schema::KeyPath;
use ds_settings::{InvalidKey, Loaded};
use std::collections::BTreeMap;

/// What `spec`'s file holds: every valid value laid over the keys' defaults. A missing file is
/// the defaults; a bad value costs only its own key (its default is used and it is reported in
/// `invalid`); a key no schema describes is reported in `unknown` and is not part of the
/// result, so the next save drops it. A file that is there and cannot be read (no permission,
/// not text, not an ordinary file, over a megabyte) is not a missing one: it shows the
/// defaults and is reported as the one invalid key of the whole file, and [`crate::save`]
/// refuses to write over it. Reading never fails.
pub fn load(home: &ConfigHome, spec: &FileSpec) -> Loaded<FileValues> {
    let path = home.path_of(&spec.file);
    let text = match text_of(&path) {
        Ok(Some(text)) => text,
        Ok(None) => return Loaded::clean(defaults(spec)),
        Err(error) => return Loaded::garbled(defaults(spec), reason_of(&error)),
    };
    match text.parse::<toml::Table>() {
        Ok(mut table) => {
            // A corner's command in its older spelling, read the way the program itself reads it.
            ds_settings::schema::fold_older_commands(&mut table);
            overlay(spec, &table)
        }
        Err(error) => Loaded::garbled(defaults(spec), error.message().to_owned()),
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
                        // The value itself is not echoed: a free-text key can hold a token.
                        reason: "the value is not one this control holds, kept as written"
                            .to_owned(),
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
        // Settings shows a value as written, so nothing here is clamped.
        clamped: Vec::new(),
    }
}
