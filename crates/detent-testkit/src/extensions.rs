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

/// Installs the Extension `id` with a folder of file icons holding one picture, `text.svg`.
///
/// # Panics
/// If the files cannot be written.
pub fn install_file_icons(data_home: &Path, id: &str, name: &str, summary: &str) -> PathBuf {
    let folder = data_home.join("quire").join("extensions").join(id);
    write(
        &folder,
        "extension.toml",
        &format!(
            "id = \"{id}\"\nname = \"{name}\"\nprotocol = 1\nsummary = \"{summary}\"\n\n\
             [[provides]]\ncapability = \"file_icons\"\ndir = \"file-icons\"\n"
        ),
    );
    write(
        &folder.join("file-icons"),
        "text.svg",
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect width=\"8\" height=\"8\"/></svg>\n",
    );
    folder
}

/// Installs the person's own Extension (`user.YourChanges`) as quire writes it, holding one
/// picture when `changes` is true, else an empty folder.
///
/// # Panics
/// If the files cannot be written.
pub fn install_your_changes(data_home: &Path, changes: Changes) -> PathBuf {
    let folder = data_home
        .join("quire")
        .join("extensions")
        .join("user.YourChanges");
    write(
        &folder,
        "extension.toml",
        "id = \"user.YourChanges\"\nname = \"Your changes\"\nprotocol = 1\nversion = \"1.0.0\"\n\
         summary = \"The icons you chose for single file types\"\n\n\
         [[provides]]\ncapability = \"file_icons\"\ndir = \"file-icons\"\n",
    );
    std::fs::create_dir_all(folder.join("file-icons")).expect("the pictures folder");
    if changes == Changes::Held {
        write(
            &folder.join("file-icons"),
            "text.svg",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect width=\"8\" height=\"8\"/></svg>\n",
        );
    }
    folder
}

/// Whether the person's own Extension holds a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Changes {
    /// One picture is set.
    Held,
    /// None is.
    None,
}
