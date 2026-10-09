//! Which keys a schema lets an agent set.
//!
//! `ds_settings::schema::KeySpec` has no such field yet (FINDINGS "A schema cannot mark a key
//! agent-settable"), so a program marks a key by writing `agent = "settable"` in its `[[key]]`
//! table, which `KeySpec`'s deserialiser ignores. This file reads that mark from the schema
//! files themselves. A key without it is hands-off: an agent may read it, never set it.

use ds_settings::schema::{AppId, KeyPath, is_never_settable};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// Whether an agent may set a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentSetting {
    /// The schema does not mark the key: only the person sets it.
    #[default]
    HandsOff,
    /// The schema marks the key `agent = "settable"`.
    Settable,
}

/// The keys of each program that its schema marks settable.
pub(crate) type Marks = HashMap<AppId, BTreeSet<KeyPath>>;

/// The marks of the schema files under `dirs`, read the way `discover` reads them: in
/// directory order, a program's first file wins, a file that does not parse is skipped.
pub(crate) fn read_marks(dirs: &[PathBuf]) -> Marks {
    dirs.iter()
        .flat_map(|dir| schema_files(dir))
        .fold(Marks::new(), |mut marks, path| {
            if let Some((app, keys)) = marked(&path) {
                marks.entry(app).or_insert(keys);
            }
            marks
        })
}

/// The `*.settings.toml` files of `dir`, in file name order.
fn schema_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".settings.toml"))
        })
        .collect();
    files.sort();
    files
}

/// The program a schema file is for and the keys it marks settable.
fn marked(path: &Path) -> Option<(AppId, BTreeSet<KeyPath>)> {
    let text = std::fs::read_to_string(path).ok()?;
    keys_marked(&text)
}

fn keys_marked(text: &str) -> Option<(AppId, BTreeSet<KeyPath>)> {
    let table: toml::Table = text.parse().ok()?;
    let app = AppId(table.get("app")?.as_str()?.to_owned());
    let keys = table
        .get("key")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_table)
        .filter(|key| key.get("agent").and_then(toml::Value::as_str) == Some("settable"))
        .filter_map(|key| key.get("path")?.as_str())
        .filter(|path| !is_never_settable(path))
        .map(|path| KeyPath(path.to_owned()))
        .collect();
    Some((app, keys))
}

#[cfg(test)]
mod tests {
    use super::keys_marked;

    const SCHEMA: &str = r#"
app = "sill"
file = "sill/settings.toml"
version = 1

[[key]]
path = "dock.magnification"
agent = "settable"

[[key]]
path = "dock.position"

[[key]]
path = "dock.hide"
agent = "hands_off"

[[key]]
path = "idle.timeout"
agent = "settable"
"#;

    #[test]
    fn only_a_key_marked_settable_is_settable() {
        let (app, keys) = keys_marked(SCHEMA).expect("a schema");
        assert_eq!(app.0, "sill");
        let paths: Vec<&str> = keys.iter().map(|k| k.0.as_str()).collect();
        assert_eq!(paths, ["dock.magnification"]);
    }

    #[test]
    fn text_that_is_not_a_schema_marks_nothing() {
        assert_eq!(keys_marked("not = [valid"), None);
        assert_eq!(keys_marked("version = 1"), None, "no app");
    }
}
