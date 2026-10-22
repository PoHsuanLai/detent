//! The pictures the Wallpaper page offers: the ones the system ships, in its backgrounds folders,
//! and the ones the person added, copied into detent's own data folder so they stay when the
//! original moves. A wallpaper package (a folder of one picture per screen size) is one picture.

use crate::packages::package_in;
use crate::values::DataHome;
use detent_model::{Picture, PictureFrom, is_picture};
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

/// The most pictures one section of a listing returns, so a folder of thousands does not stall
/// the page, and the system's can never push the person's own out.
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

    /// The folders the pictures are in: the system's, then the added ones. What the window may
    /// read pictures from is these and nothing else.
    pub fn folders(&self) -> Vec<PathBuf> {
        self.system.iter().chain([&self.added]).cloned().collect()
    }

    /// Every picture in the folders, the system's first, each folder in name order, each marked
    /// with whose folder it is in. The system's and the added ones are each cut at the most.
    pub fn list(&self) -> Vec<Picture> {
        let mut system = Vec::new();
        for folder in &self.system {
            walk(folder, DEPTH, PictureFrom::System, &mut system);
        }
        system.truncate(MOST);
        let mut added = Vec::new();
        walk(&self.added, DEPTH, PictureFrom::Added, &mut added);
        added.truncate(MOST);
        system.append(&mut added);
        system
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

/// The pictures in `folder` and the folders under it down to `depth`, marked `from` and appended
/// to `found`. A
/// wallpaper package is taken whole and not looked into.
fn walk(folder: &Path, depth: usize, from: PictureFrom, found: &mut Vec<Picture>) {
    let Ok(read) = std::fs::read_dir(folder) else {
        return;
    };
    let mut entries: Vec<PathBuf> = read.filter_map(|entry| Some(entry.ok()?.path())).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            match package_in(&path) {
                Some(package) => found.push(package.placed(from)),
                None if depth > 1 => walk(&path, depth - 1, from, found),
                None => {}
            }
        } else if path.is_file() && is_picture(&path) {
            found.push(Picture::file(path).placed(from));
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
        let used = |candidate: &str| taken.contains(&candidate);
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
