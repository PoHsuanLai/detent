//! Keys a file sets that no schema describes.

use ds_settings::UnknownKey;
use ds_settings::schema::{ForeignTables, KeySpec};

/// A leaf of a TOML table with its key path.
fn leaves(table: &toml::Table, prefix: &[String]) -> Vec<Vec<String>> {
    table
        .iter()
        .flat_map(|(key, value)| {
            let path: Vec<String> = prefix.iter().cloned().chain([key.clone()]).collect();
            match value {
                toml::Value::Table(inner) if !inner.is_empty() => leaves(inner, &path),
                _ => vec![path],
            }
        })
        .collect()
}

fn segments(spec: &KeySpec) -> Vec<&str> {
    spec.path.0.split('.').collect()
}

/// The keys of `table` no spec in `specs` reads, each cut at the first table the schema has no
/// key for (a whole unknown table is one entry). The top-level `version` is the file's own
/// and is not reported, nor is any leaf under a table the program owns (`foreign`).
pub(crate) fn unknown_keys(
    table: &toml::Table,
    specs: &[KeySpec],
    foreign: &ForeignTables,
) -> Vec<UnknownKey> {
    let known: Vec<Vec<&str>> = specs.iter().map(segments).collect();
    let mut found: Vec<UnknownKey> = Vec::new();
    for leaf in leaves(table, &[]) {
        if leaf.as_slice() == ["version"] || foreign.holds(&leaf.join(".")) {
            continue;
        }
        let cut = (1..=leaf.len()).find(|&len| {
            !known
                .iter()
                .any(|path| path.len() >= len && path[..len].iter().zip(&leaf).all(|(a, b)| a == b))
        });
        // A leaf inside a known key (a table-valued value) is that key's own.
        let inside_known = known
            .iter()
            .any(|path| path.len() <= leaf.len() && path.iter().zip(&leaf).all(|(a, b)| a == b));
        if let (Some(len), false) = (cut, inside_known) {
            let key = UnknownKey(leaf[..len].join("."));
            if !found.contains(&key) {
                found.push(key);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::unknown_keys;
    use ds_settings::UnknownKey;
    use ds_settings::schema::ForeignTables;

    fn tables(names: &[&str]) -> ForeignTables {
        ForeignTables(names.iter().map(|n| (*n).to_owned()).collect())
    }

    #[test]
    fn a_foreign_table_is_left_alone_and_a_stray_key_beside_it_is_not() {
        let table: toml::Table = toml::from_str(
            "version = 1\nstray = 1\n[engines.local]\ncmd = \"x\"\n[probe]\nms = 5\n",
        )
        .expect("toml");
        let cases = [
            (tables(&[]), vec!["engines", "probe", "stray"]),
            (tables(&["engines", "probe"]), vec!["stray"]),
            (tables(&["engines"]), vec!["probe", "stray"]),
        ];
        for (foreign, expected) in cases {
            let found = unknown_keys(&table, &[], &foreign);
            let expected: Vec<UnknownKey> = expected
                .into_iter()
                .map(|k| UnknownKey(k.to_owned()))
                .collect();
            let mut found = found;
            found.sort_by(|a, b| a.0.cmp(&b.0));
            assert_eq!(found, expected, "{foreign:?}");
        }
    }
}
