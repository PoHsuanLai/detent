//! The Wallpaper page: the page quire's schemas declare as `wallpaper`, and the pictures its grid
//! shows. The page holds one key, `wallpaper.picture`, drawn by the picture control.

use ds_settings::schema::Page;
use std::path::{Path, PathBuf};

/// The page's slug: what `--page` and an `open-page` action take.
pub(crate) const SLUG: &str = "wallpaper";

/// Whether `page` is the Wallpaper page.
pub fn is_wallpaper(page: &Page) -> bool {
    matches!(page, Page::App(id) if id == SLUG)
}

/// What the desktop can draw as a wallpaper (png and jpeg), by the end of the file's name.
const PICTURE_ENDINGS: &[&str] = &["png", "jpg", "jpeg"];

/// Whether `path` names a picture the desktop can draw.
pub fn is_picture(path: &Path) -> bool {
    path.extension()
        .and_then(|ending| ending.to_str())
        .is_some_and(|ending| {
            PICTURE_ENDINGS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(ending))
        })
}

/// Whether a picture is the one on the desktop now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PictureMark {
    /// It is.
    Current,
    /// It is not.
    Other,
}

/// One picture of the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureTile {
    /// Where the file is: the value the key stores.
    pub path: PathBuf,
    /// Its name under the picture: the file's name without its ending, words apart.
    pub name: String,
    /// Whether it is the current one.
    pub mark: PictureMark,
}

/// The words of a file's name: the ending off, `_` and `-` read as spaces.
fn name_of(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().replace(['_', '-'], " "))
        .unwrap_or_default()
}

/// The grid for the pictures `found`, sorted by name, with `current` marked. The current picture
/// is always in the grid, even when it lies outside every folder looked in, so the person sees
/// what is on the desktop; an empty `current` marks none.
pub fn picture_tiles(found: &[PathBuf], current: &str) -> Vec<PictureTile> {
    let current_path = (!current.is_empty()).then(|| PathBuf::from(current));
    let mut paths: Vec<PathBuf> = found
        .iter()
        .filter(|path| is_picture(path))
        .cloned()
        .collect();
    paths.extend(
        current_path
            .iter()
            .filter(|path| !paths.contains(path))
            .cloned()
            .collect::<Vec<_>>(),
    );
    paths.sort_by_key(|path| (name_of(path).to_lowercase(), path.clone()));
    paths.dedup();
    paths
        .into_iter()
        .map(|path| PictureTile {
            mark: if current_path.as_ref() == Some(&path) {
                PictureMark::Current
            } else {
                PictureMark::Other
            },
            name: name_of(&path),
            path,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PictureMark, is_picture, is_wallpaper, picture_tiles};
    use ds_settings::schema::Page;
    use std::path::{Path, PathBuf};

    #[test]
    fn only_png_and_jpeg_files_are_pictures() {
        let cases: &[(&str, bool)] = &[
            ("/a/dune.png", true),
            ("/a/dune.JPG", true),
            ("/a/dune.jpeg", true),
            ("/a/dune.webp", false),
            ("/a/dune", false),
            ("/a/.png", false),
        ];
        for (path, want) in cases {
            assert_eq!(is_picture(Path::new(path)), *want, "{path}");
        }
    }

    #[test]
    fn the_declared_wallpaper_page_is_recognised() {
        assert!(is_wallpaper(&Page::declared("wallpaper")));
        assert!(!is_wallpaper(&Page::Appearance));
        assert!(!is_wallpaper(&Page::declared("displays")));
    }

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    fn summary(found: &[&str], current: &str) -> Vec<(String, PictureMark)> {
        picture_tiles(&paths(found), current)
            .into_iter()
            .map(|tile| (tile.name, tile.mark))
            .collect()
    }

    #[test]
    fn the_grid_is_sorted_marks_the_current_one_and_always_holds_it() {
        use PictureMark::{Current, Other};
        let cases: &[(&str, &[&str], &str, &[(&str, PictureMark)])] = &[
            ("none", &[], "", &[]),
            (
                "sorted by name, nothing current",
                &["/s/b_sky.jpg", "/s/a-dune.png", "/s/notes.txt"],
                "",
                &[("a dune", Other), ("b sky", Other)],
            ),
            (
                "the current one is marked",
                &["/s/b_sky.jpg", "/s/a-dune.png"],
                "/s/b_sky.jpg",
                &[("a dune", Other), ("b sky", Current)],
            ),
            (
                "the current one outside the folders is added",
                &["/s/a-dune.png"],
                "/home/p/mine.png",
                &[("a dune", Other), ("mine", Current)],
            ),
        ];
        for (case, found, current, want) in cases {
            assert_eq!(&summary(found, current), want, "{case}");
        }
    }
}
