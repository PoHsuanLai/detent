//! The pictures the Wallpaper page offers, over scratch folders.

use detent_files::PictureDirs;
use detent_model::Picture;
use detent_testkit::Scratch;
use std::fs;
use std::path::{Path, PathBuf};

fn paths(dirs: &PictureDirs) -> Vec<PathBuf> {
    dirs.list()
        .into_iter()
        .map(|picture| picture.path)
        .collect()
}

fn touch(path: &PathBuf, bytes: &[u8]) {
    fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
    fs::write(path, bytes).expect("write");
}

#[test]
fn the_system_pictures_come_first_and_an_added_one_is_kept_under_its_own_name() {
    let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "pictures");
    let system = scratch.join("backgrounds");
    touch(&system.join("b.jpg"), b"b");
    touch(&system.join("release/a.png"), b"a");
    touch(&system.join("notes.txt"), b"x");
    let dirs = PictureDirs::new(vec![system.clone()], scratch.join("added"));
    assert_eq!(
        paths(&dirs),
        [system.join("b.jpg"), system.join("release/a.png")]
    );
    assert_eq!(
        dirs.folders(),
        [system.clone(), scratch.join("added")],
        "the folders are the system's, then the added one"
    );

    let mine = scratch.join("Downloads/b.jpg");
    touch(&mine, b"mine");
    let kept = dirs.add(&mine).expect("added");
    assert_eq!(kept, scratch.join("added/b.jpg"));
    assert_eq!(fs::read(&kept).expect("copied"), b"mine");
    let again = scratch.join("Elsewhere/b.jpg");
    touch(&again, b"other");
    assert_eq!(
        dirs.add(&again).expect("added"),
        scratch.join("added/b 2.jpg"),
        "a second picture of the same name does not replace the first"
    );
    assert_eq!(
        dirs.add(&system.join("b.jpg")).expect("in place"),
        system.join("b.jpg"),
        "a picture already in a folder is not copied"
    );
    assert_eq!(dirs.list().len(), 4);
}

/// A package named `title` (none: no `KPlugin.Name`) with the size files `sizes` under
/// `contents/images`, a dark one, and a screenshot when `shot` names its file.
fn package(root: &Path, title: Option<&str>, sizes: &[&str], shot: Option<&str>) {
    let metadata = match title {
        Some(title) => format!(r#"{{"KPlugin": {{"Name": "{title}", "Id": "x"}}}}"#),
        None => "{}".to_owned(),
    };
    touch(&root.join("metadata.json"), metadata.as_bytes());
    for size in sizes {
        touch(&root.join("contents/images").join(size), b"p");
    }
    touch(&root.join("contents/images_dark/9999x9999.png"), b"d");
    if let Some(shot) = shot {
        touch(&root.join("contents").join(shot), b"s");
    }
}

#[test]
fn a_wallpaper_package_is_one_picture_and_a_plain_folder_stays_its_files() {
    // name, title, sizes, screenshot, (stored size file, title, preview file)
    type Case<'a> = (
        &'a str,
        Option<&'a str>,
        &'a [&'a str],
        Option<&'a str>,
        Option<(&'a str, &'a str, Option<&'a str>)>,
    );
    let cases: &[Case] = &[
        (
            "Dune",
            Some("Dune Dusk"),
            &["1920x1080.png", "5120x2880.png", "3840x2160.jpg"],
            Some("screenshot.png"),
            Some((
                "contents/images/5120x2880.png",
                "Dune Dusk",
                Some("contents/screenshot.png"),
            )),
        ),
        (
            "Quiet",
            None,
            &["100x100.png", "2560x1440.png"],
            None,
            Some(("contents/images/2560x1440.png", "Quiet", None)),
        ),
        (
            "Odd",
            Some("Odd"),
            &["wide.png", "wider.jpg"],
            None,
            Some(("contents/images/wider.jpg", "Odd", None)),
        ),
    ];
    for (folder, title, sizes, shot, want) in cases {
        let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "package");
        let system = scratch.join("wallpapers");
        let root = system.join(folder);
        package(&root, *title, sizes, *shot);
        let dirs = PictureDirs::new(vec![system.clone()], scratch.join("added"));
        let got: Vec<Picture> = dirs.list();
        let want: Vec<Picture> = match want {
            Some((image, title, preview)) => vec![Picture::packaged(
                root.join(image),
                (*title).to_owned(),
                preview.map(|preview| root.join(preview)),
            )],
            None => Vec::new(),
        };
        assert_eq!(got, want, "{folder}");
    }
}
