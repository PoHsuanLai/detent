//! Extensions as a package would install them, into a scratch data home.

use std::path::{Path, PathBuf};

/// The manifest of an Extension with a look, naming `look.css` beside it.
fn look_manifest(id: &str, name: &str, summary: &str) -> String {
    format!(
        "id = \"{id}\"\nname = \"{name}\"\nprotocol = 1\nsummary = \"{summary}\"\n\n\
         [[provides]]\ncapability = \"look\"\nstylesheet = \"look.css\"\nstyles_for = 3\n"
    )
}

/// Writes `text` at `folder/file`, the folder made.
fn write(folder: &Path, file: &str, text: &str) {
    std::fs::create_dir_all(folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display()));
    let path = folder.join(file);
    std::fs::write(&path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

/// Installs the Extension `id` with a look and its stylesheet into `data_home`
/// (`quire/extensions/<id>`), and returns its folder.
///
/// # Panics
/// If the files cannot be written.
pub fn install_look(data_home: &Path, id: &str, name: &str, summary: &str) -> PathBuf {
    let folder = data_home.join("quire").join("extensions").join(id);
    write(&folder, "extension.toml", &look_manifest(id, name, summary));
    write(&folder, "look.css", ".ds { --accent: red; }\n");
    folder
}

/// Installs the Extension `id` with a look whose stylesheet is not there, so it cannot be used.
///
/// # Panics
/// If the manifest cannot be written.
pub fn install_torn_look(data_home: &Path, id: &str, name: &str) -> PathBuf {
    let folder = data_home.join("quire").join("extensions").join(id);
    write(&folder, "extension.toml", &look_manifest(id, name, "Torn"));
    folder
}
