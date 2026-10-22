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

/// What the desktop can draw as a wallpaper (png, jpeg and webp), by the end of the file's name.
const PICTURE_ENDINGS: &[&str] = &["png", "jpg", "jpeg", "webp"];

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

/// Whose folder a picture lies in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PictureFrom {
    /// The system's backgrounds folders.
    #[default]
    System,
    /// The folder of the pictures the person added.
    Added,
}

/// One picture on offer, as the folders list it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// Whose folder it lies in.
    pub from: PictureFrom,
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
            from: PictureFrom::System,
            path,
            title: None,
            preview: None,
        }
    }

    /// This picture, listed from `from`'s folder (a new one is the system's until placed).
    pub fn placed(self, from: PictureFrom) -> Picture {
        Picture { from, ..self }
    }

    /// This picture under `title`, drawn small from `preview`.
    pub fn packaged(path: PathBuf, title: String, preview: Option<PathBuf>) -> Picture {
        Picture {
            from: PictureFrom::System,
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

/// The tile of `picture`, marked when its path is `current`.
fn tile_of(picture: Picture, current: Option<&PathBuf>) -> PictureTile {
    PictureTile {
        mark: if current == Some(&picture.path) {
            PictureMark::Current
        } else {
            PictureMark::Other
        },
        name: picture.title.unwrap_or_else(|| name_of(&picture.path)),
        preview: picture.preview.unwrap_or_else(|| picture.path.clone()),
        path: picture.path,
    }
}

/// `tiles` sorted by name, a path once.
fn sorted(mut tiles: Vec<PictureTile>) -> Vec<PictureTile> {
    tiles.sort_by_key(|tile| (tile.name.to_lowercase(), tile.path.clone()));
    tiles.dedup_by(|later, earlier| later.path == earlier.path);
    tiles
}

/// What the Wallpaper page shows: the picture on the desktop, then the person's own pictures,
/// then the system's.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PictureSections {
    /// The picture on the desktop now, listed or not; none when no picture is stored.
    pub current: Option<PictureTile>,
    /// The pictures the person added, by name.
    pub added: Vec<PictureTile>,
    /// The system's pictures, by name.
    pub system: Vec<PictureTile>,
}

/// The place of the current tile among `tiles`, or `None` when it is not one of them.
pub fn current_place(tiles: &[PictureTile]) -> Option<usize> {
    tiles
        .iter()
        .position(|tile| tile.mark == PictureMark::Current)
}

/// The sections for the pictures `found`, with `current` marked. The current picture is always
/// the section's `current`, even when it lies outside every folder looked in, so the person sees
/// what is on the desktop; an empty `current` marks none. A picture the person added is not
/// offered twice among the system's.
pub fn picture_sections(found: &[Picture], current: &str) -> PictureSections {
    let current_path = (!current.is_empty()).then(|| PathBuf::from(current));
    let pictures: Vec<Picture> = found
        .iter()
        .filter(|picture| is_picture(&picture.path))
        .cloned()
        .collect();
    let listed = current_path.as_ref().and_then(|path| {
        pictures
            .iter()
            .find(|picture| &picture.path == path)
            .cloned()
    });
    let current_tile = current_path
        .as_ref()
        .map(|path| listed.unwrap_or_else(|| Picture::file(path.clone())))
        .map(|picture| tile_of(picture, current_path.as_ref()));
    let in_section = |from: PictureFrom| {
        sorted(
            pictures
                .iter()
                .filter(|picture| picture.from == from)
                .cloned()
                .map(|picture| tile_of(picture, current_path.as_ref()))
                .collect(),
        )
    };
    let added = in_section(PictureFrom::Added);
    let system = in_section(PictureFrom::System)
        .into_iter()
        .filter(|tile| !added.iter().any(|mine| mine.path == tile.path))
        .collect();
    PictureSections {
        current: current_tile,
        added,
        system,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Picture, PictureFrom, PictureMark, current_place, is_picture, is_wallpaper,
        picture_sections,
    };
    use ds_settings::schema::Page;
    use std::path::{Path, PathBuf};

    #[test]
    fn only_png_jpeg_and_webp_files_are_pictures() {
        let cases: &[(&str, bool)] = &[
            ("/a/dune.png", true),
            ("/a/dune.JPG", true),
            ("/a/dune.jpeg", true),
            ("/a/dune.webp", true),
            ("/a/dune.heic", false),
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

    fn listed(system: &[&str], added: &[&str]) -> Vec<Picture> {
        let from = |paths: &[&str], from| -> Vec<Picture> {
            paths
                .iter()
                .map(|path| Picture::file(PathBuf::from(path)).placed(from))
                .collect()
        };
        [
            from(system, PictureFrom::System),
            from(added, PictureFrom::Added),
        ]
        .concat()
    }

    fn names(tiles: &[super::PictureTile]) -> Vec<&str> {
        tiles.iter().map(|tile| tile.name.as_str()).collect()
    }

    #[test]
    fn the_sections_are_sorted_split_by_origin_and_the_current_one_is_always_known() {
        let found = listed(
            &[
                "/s/b_sky.jpg",
                "/s/a-dune.png",
                "/s/notes.txt",
                "/s/mine.png",
            ],
            &["/h/z.webp", "/h/mine.png"],
        );

        let none = picture_sections(&found, "");
        assert_eq!(names(&none.system), ["a dune", "b sky", "mine"]);
        assert_eq!(names(&none.added), ["mine", "z"]);
        assert_eq!(none.current, None);

        let added = picture_sections(&found, "/h/z.webp");
        assert_eq!(
            added.current.as_ref().map(|tile| tile.name.as_str()),
            Some("z")
        );
        assert_eq!(current_place(&added.added), Some(1));
        assert_eq!(
            current_place(&added.system),
            None,
            "the other section has no cursor"
        );

        let outside = picture_sections(&found, "/home/p/elsewhere.png");
        let tile = outside.current.expect("the current one is known");
        assert_eq!(
            (tile.name.as_str(), tile.mark),
            ("elsewhere", PictureMark::Current)
        );
        assert_eq!(
            current_place(&outside.added).or(current_place(&outside.system)),
            None
        );
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
        let sections = picture_sections(&found, "");
        let got: Vec<(&str, &str)> = sections
            .system
            .iter()
            .map(|tile| {
                (
                    tile.name.as_str(),
                    tile.preview.to_str().unwrap_or_default(),
                )
            })
            .collect();
        // "a sky" sorts before "dune" (case-insensitive).
        assert_eq!(
            got,
            [
                ("a sky", "/s/a_sky.png"),
                ("Dune", "/w/Dune/contents/screenshot.png")
            ]
        );
    }
}
