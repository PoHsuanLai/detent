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

/// One picture on offer, as the folders list it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// The file the key stores.
    pub path: PathBuf,
    /// The name it carries itself (a wallpaper package names its picture); none: the file's name.
    pub title: Option<String>,
    /// The file drawn small in its tile when that is not `path` itself (a package's screenshot).
    pub preview: Option<PathBuf>,
}

impl Picture {
    /// A plain picture file: named by its file, drawn from itself.
    pub fn file(path: PathBuf) -> Picture {
        Picture {
            path,
            title: None,
            preview: None,
        }
    }

    /// This picture under `title`, drawn small from `preview`.
    pub fn packaged(path: PathBuf, title: String, preview: Option<PathBuf>) -> Picture {
        Picture {
            path,
            title: Some(title),
            preview,
        }
    }
}

/// One picture of the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PictureTile {
    /// Where the file is: the value the key stores.
    pub path: PathBuf,
    /// Its name under the picture: its own title, else the file's name without its ending, words
    /// apart.
    pub name: String,
    /// The file the tile's small picture is made from.
    pub preview: PathBuf,
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
pub fn picture_tiles(found: &[Picture], current: &str) -> Vec<PictureTile> {
    let current_path = (!current.is_empty()).then(|| PathBuf::from(current));
    let mut pictures: Vec<Picture> = found
        .iter()
        .filter(|picture| is_picture(&picture.path))
        .cloned()
        .collect();
    if let Some(path) = &current_path
        && !pictures.iter().any(|picture| &picture.path == path)
    {
        pictures.push(Picture::file(path.clone()));
    }
    let mut tiles: Vec<PictureTile> = pictures
        .into_iter()
        .map(|picture| PictureTile {
            mark: if current_path.as_ref() == Some(&picture.path) {
                PictureMark::Current
            } else {
                PictureMark::Other
            },
            name: picture.title.unwrap_or_else(|| name_of(&picture.path)),
            preview: picture.preview.unwrap_or_else(|| picture.path.clone()),
            path: picture.path,
        })
        .collect();
    tiles.sort_by_key(|tile| (tile.name.to_lowercase(), tile.path.clone()));
    tiles.dedup_by(|later, earlier| later.path == earlier.path);
    tiles
}

#[cfg(test)]
mod tests {
    use super::{Picture, PictureMark, is_picture, is_wallpaper, picture_tiles};
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

    fn paths(list: &[&str]) -> Vec<Picture> {
        list.iter()
            .map(|path| Picture::file(PathBuf::from(path)))
            .collect()
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
        type Case<'a> = (
            &'a str,
            &'a [&'a str],
            &'a str,
            &'a [(&'a str, PictureMark)],
        );
        let cases: &[Case] = &[
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
            let got = summary(found, current);
            let got: Vec<(&str, PictureMark)> = got
                .iter()
                .map(|(name, mark)| (name.as_str(), *mark))
                .collect();
            assert_eq!(got, *want, "{case}");
        }
    }

    #[test]
    fn a_packaged_picture_keeps_its_title_and_is_drawn_from_its_preview() {
        let found = [
            Picture::packaged(
                PathBuf::from("/w/Dune/contents/images/5120x2880.png"),
                "Dune".to_owned(),
                Some(PathBuf::from("/w/Dune/contents/screenshot.png")),
            ),
            Picture::file(PathBuf::from("/s/a_sky.png")),
        ];
        let tiles = picture_tiles(&found, "");
        let got: Vec<(&str, &str)> = tiles
            .iter()
            .map(|tile| {
                (
                    tile.name.as_str(),
                    tile.preview.to_str().unwrap_or_default(),
                )
            })
            .collect();
        let want = [
            ("Dune", "/w/Dune/contents/screenshot.png"),
            ("a sky", "/s/a_sky.png"),
        ];
        // "a sky" sorts before "dune" (case-insensitive).
        assert_eq!(got, [want[1], want[0]]);
    }
}
