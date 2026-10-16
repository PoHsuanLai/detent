//! Where desktop entries are: the `.desktop` files under some folders, and the id a file's place
//! gives its app.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The folder a desktop entry's id is counted from: `applications/kde/okular.desktop` is
/// `kde-okular`.
const APPLICATIONS: &str = "/applications/";

/// What a folder holds that the walk goes on with.
enum Item {
    File(PathBuf),
    Folder(PathBuf),
}

/// What `path` is to the walk: a folder (by its real path, so a link back up the tree is seen for
/// what it is), a `.desktop` file, or nothing.
fn classify(path: PathBuf) -> Option<Item> {
    let kind = std::fs::metadata(&path).ok()?;
    if kind.is_dir() {
        return path.canonicalize().ok().map(Item::Folder);
    }
    let is_entry = kind.is_file() && path.extension().is_some_and(|ext| ext == "desktop");
    is_entry.then_some(Item::File(path))
}

/// What `dir` holds, by name, so the order never depends on the disk. A folder that cannot be
/// read holds nothing.
fn children(dir: &Path) -> Vec<Item> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = read.filter_map(Result::ok).map(|e| e.path()).collect();
    paths.sort_unstable();
    paths.into_iter().filter_map(classify).collect()
}

/// Every `.desktop` file under `dirs`, folders in the order given and a folder's own files before
/// the folders inside it. A folder that is not there is skipped, and a folder reached twice
/// (through a link) is walked once.
pub(crate) fn desktop_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    // The next folder is the last one; a folder's subfolders go on top, so they are walked
    // before the folders that were given after it.
    let mut pending: Vec<PathBuf> = dirs.iter().rev().cloned().collect();
    let mut seen = BTreeSet::new();
    let mut found = Vec::new();
    while let Some(dir) = pending.pop() {
        let dir = dir.canonicalize().unwrap_or(dir);
        seen.insert(dir.clone());
        for item in children(&dir) {
            match item {
                Item::File(file) => found.push(file),
                Item::Folder(folder) => {
                    if seen.insert(folder.clone()) {
                        pending.push(folder);
                    }
                }
            }
        }
    }
    found
}

/// The id of the app the entry at `path` is: the name under the last `applications` folder with
/// its folders joined by `-`, else the file's name; none for a path that is not a `.desktop` file
/// or not text.
pub(crate) fn app_id(path: &Path) -> Option<String> {
    let full = path.to_str()?.strip_suffix(".desktop")?;
    let id = match full.rsplit_once(APPLICATIONS) {
        Some((_, under)) => under.replace('/', "-"),
        None => full.rsplit('/').next().unwrap_or(full).to_owned(),
    };
    (!id.is_empty()).then_some(id)
}

#[cfg(test)]
mod tests {
    use super::app_id;
    use std::path::Path;

    #[test]
    fn a_path_gives_the_id_of_its_app() {
        // `(path, id)`.
        const CASES: &[(&str, Option<&str>)] = &[
            ("/usr/share/applications/grim.desktop", Some("grim")),
            (
                "/usr/share/applications/kde/okular.desktop",
                Some("kde-okular"),
            ),
            ("/home/a/.local/share/applications/x.desktop", Some("x")),
            (
                "/applications/old/applications/deep/x.desktop",
                Some("deep-x"),
            ),
            ("/opt/apps/tool.desktop", Some("tool")),
            ("tool.desktop", Some("tool")),
            ("/opt/apps/org.gnome.Loupe.desktop", Some("org.gnome.Loupe")),
            ("/opt/apps/tool.txt", None),
            ("/opt/apps/tool", None),
            ("/usr/share/applications/.desktop", None),
        ];
        for (path, want) in CASES {
            assert_eq!(app_id(Path::new(path)).as_deref(), *want, "{path}");
        }
    }
}
