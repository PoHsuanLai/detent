//! The intents manifests of the installed apps: one `<AppName>.toml` for each app in a
//! `quire/intents` folder of a data directory. The Hot Corners sheet reads them to list the
//! actions a corner can run; what they mean is `detent-model`'s.

use ds_helpers::{ReadCap, read_text_capped};
use std::path::{Path, PathBuf};

/// The most a manifest may hold. An app's declaration is a few dozen KiB at most.
const MANIFEST_CAP: ReadCap = ReadCap::mib(1);

/// The `.toml` files directly in `dir`, by name, so the order never depends on the disk. A folder
/// that is not there or cannot be read holds none.
fn manifest_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = read
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml") && path.is_file())
        .collect();
    files.sort_unstable();
    files
}

/// The text of every manifest under `dirs`, folders in the order given (the order of trust) and
/// files by name. A file that cannot be read, is over the cap or is not text is skipped.
pub fn read_manifests(dirs: &[PathBuf]) -> Vec<String> {
    dirs.iter()
        .flat_map(|dir| manifest_files(dir))
        .filter_map(|file| read_text_capped(&file, MANIFEST_CAP).ok())
        .collect()
}
