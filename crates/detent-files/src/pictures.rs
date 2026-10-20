//! The pictures the Wallpaper page offers: the ones the system ships, in its backgrounds folders,
//! and the ones the person added, copied into detent's own data folder so they stay when the
//! original moves.

use crate::values::DataHome;
use detent_model::is_picture;
use std::io;
use std::path::{Path, PathBuf};

/// Where the system keeps its wallpapers.
const SYSTEM_FOLDERS: &[&str] = &[
    "/usr/share/backgrounds",
    "/usr/local/share/backgrounds",
    "/usr/share/wallpapers",
];

/// The folder of the added pictures, under `<data home>/detent/`.
const ADDED_FOLDER: &str = "pictures";

/// How deep a folder of backgrounds is looked into (a release's folder holds a folder of sizes).
const DEPTH: usize = 3;

/// The most pictures one listing returns, so a folder of thousands does not stall the page.
const MOST: usize = 400;

/// The folders the pictures are in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureDirs {
    system: Vec<PathBuf>,
    added: PathBuf,
}

impl PictureDirs {
    /// These folders: `system` ones, and `added` for the person's.
    pub fn new(system: Vec<PathBuf>, added: PathBuf) -> PictureDirs {
        PictureDirs { system, added }
    }

    /// This computer's folders, with the added pictures under `home`.
    pub fn standard(home: &DataHome) -> PictureDirs {
        PictureDirs::new(
            SYSTEM_FOLDERS.iter().map(PathBuf::from).collect(),
            home.0.join("detent").join(ADDED_FOLDER),
        )
    }

    /// Every picture in the folders, the system's first, each folder in name order.
    pub fn list(&self) -> Vec<PathBuf> {
        let mut found = Vec::new();
        for folder in self.system.iter().chain([&self.added]) {
            walk(folder, DEPTH, &mut found);
        }
        found.truncate(MOST);
        found
    }

    /// `source` among the pictures: where it is when it already lies in one of the folders, else a
    /// copy of it in the added folder, under a name no other picture there has.
    pub fn add(&self, source: &Path) -> io::Result<PathBuf> {
        if self
            .system
            .iter()
            .chain([&self.added])
            .any(|f| source.starts_with(f))
        {
            return Ok(source.to_path_buf());
        }
        let name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "a picture has a name"))?;
        std::fs::create_dir_all(&self.added)?;
        let kept = unique_name(name, |candidate| self.added.join(candidate).exists());
        let target = self.added.join(kept);
        std::fs::copy(source, &target)?;
        Ok(target)
    }
}

/// The pictures in `folder` and the folders under it down to `depth`, appended to `found`.
fn walk(folder: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(folder) else {
        return;
    };
    let mut entries: Vec<PathBuf> = read.filter_map(|entry| Some(entry.ok()?.path())).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() && depth > 1 {
            walk(&path, depth - 1, found);
        } else if path.is_file() && is_picture(&path) {
            found.push(path);
        }
    }
}

/// `name`, or `name` with ` 2`, ` 3`... before its ending, whichever `taken` does not say is used.
fn unique_name(name: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(name) {
        return name.to_owned();
    }
    let path = Path::new(name);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(name);
    let ending = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    (2..)
        .map(|number| format!("{stem} {number}{ending}"))
        .find(|candidate| !taken(candidate))
        .unwrap_or_else(|| name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::unique_name;

    #[test]
    fn a_taken_name_gets_the_next_free_number_before_its_ending() {
        let taken = ["dune.png", "dune 2.png"];
        let used = |candidate: &str| taken.iter().any(|name| *name == candidate);
        let cases: &[(&str, &str)] = &[
            ("sky.png", "sky.png"),
            ("dune.png", "dune 3.png"),
            ("dune 2.png", "dune 2 2.png"),
        ];
        for (name, want) in cases {
            assert_eq!(unique_name(name, used), *want, "{name}");
        }
    }
}
