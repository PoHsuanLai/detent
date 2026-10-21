//! The small pictures of the Wallpaper page, over a scratch cache: made once from the original,
//! scaled to the size kept, found again the next time, refused for what is not a picture.

use detent_files::CacheHome;
use detent_testkit::Scratch;
use detent_ui::{ThumbFault, Thumbnails};
use std::path::Path;

/// A picture of `width` by `height` pixels at `path`, in the ending's format.
fn picture(path: &Path, width: u32, height: u32) {
    std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
    image::RgbImage::from_pixel(width, height, image::Rgb([200, 90, 40]))
        .save(path)
        .expect("saved");
}

/// Width and height in pixels.
type Size = (u32, u32);

#[test]
fn a_wallpaper_is_made_small_once_inside_the_cache_folder() {
    // name, original size, size kept
    let cases: &[(&str, Size, Size)] = &[
        ("wide.png", (600, 300), (256, 128)),
        ("tall.jpg", (300, 600), (128, 256)),
    ];
    for (name, original, kept) in cases {
        let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "thumbs-make");
        let source = scratch.join("backgrounds").join(name);
        picture(&source, original.0, original.1);
        let cache = CacheHome(scratch.join("cache"));
        let thumbs = Thumbnails::under(&cache);

        let small = thumbs.of(&source).expect("made small");
        assert!(small.starts_with(thumbs.folder()), "{name}: {small:?}");
        let made = image::open(&small).expect("a picture");
        assert_eq!((made.width(), made.height()), *kept, "{name}");

        let before = std::fs::metadata(&small)
            .and_then(|m| m.modified())
            .expect("mtime");
        assert_eq!(thumbs.of(&source).expect("found again"), small, "{name}");
        let after = std::fs::metadata(&small)
            .and_then(|m| m.modified())
            .expect("mtime");
        assert_eq!(before, after, "{name}: the second look wrote nothing");
    }
}

#[test]
fn what_is_not_a_picture_or_not_a_full_path_has_no_small_picture() {
    let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "thumbs-refuse");
    let notes = scratch.join("backgrounds/notes.png");
    std::fs::create_dir_all(notes.parent().expect("a parent")).expect("mkdir");
    std::fs::write(&notes, "just words, not a picture").expect("write");
    let thumbs = Thumbnails::under(&CacheHome(scratch.join("cache")));

    assert!(thumbs.of(&notes).is_err(), "words named like a picture");
    assert!(
        thumbs.of(&scratch.join("backgrounds/missing.png")).is_err(),
        "a file that is not there"
    );
    assert!(
        matches!(
            thumbs.of(Path::new("relative.png")),
            Err(ThumbFault::NotAbsolute)
        ),
        "a path with no folder"
    );
}
