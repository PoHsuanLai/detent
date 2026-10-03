//! The committed schema fixtures: real schemas, as the programs write them.

use std::path::{Path, PathBuf};

/// `fixtures/schemas` of this repo.
pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/schemas")
}

/// Copies every fixture schema into `data_home/quire/settings`, the place discovery looks, and
/// returns that directory.
///
/// # Panics
/// If a fixture cannot be copied.
pub fn seed_schemas(data_home: &Path) -> PathBuf {
    let target = data_home.join("quire").join("settings");
    std::fs::create_dir_all(&target).unwrap_or_else(|e| panic!("{}: {e}", target.display()));
    let entries = std::fs::read_dir(fixture_dir()).unwrap_or_else(|e| panic!("fixtures: {e}"));
    for entry in entries.filter_map(Result::ok) {
        let from = entry.path();
        if let Some(name) = from.file_name() {
            std::fs::copy(&from, target.join(name))
                .unwrap_or_else(|e| panic!("{}: {e}", from.display()));
        }
    }
    target
}

/// [`seed_schemas`], with each of `paths` marked `agent = "settable"` in the schema that
/// describes it, as a program's schema would mark a key an agent may set.
///
/// # Panics
/// If a fixture cannot be copied, or no fixture describes one of `paths`.
pub fn seed_schemas_settable(data_home: &Path, paths: &[&str]) -> PathBuf {
    let target = seed_schemas(data_home);
    for path in paths {
        let line = format!("path = \"{path}\"");
        let mut marked = false;
        for entry in std::fs::read_dir(&target)
            .unwrap_or_else(|e| panic!("{}: {e}", target.display()))
            .filter_map(Result::ok)
        {
            let file = entry.path();
            let text = std::fs::read_to_string(&file).unwrap_or_default();
            if text.contains(&line) {
                let with = text.replacen(&line, &format!("{line}\nagent = \"settable\""), 1);
                std::fs::write(&file, with).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
                marked = true;
            }
        }
        assert!(marked, "no fixture schema describes {path}");
    }
    target
}
