//! The pictures the Wallpaper page offers, over scratch folders.

use detent_files::PictureDirs;
use detent_testkit::Scratch;
use std::fs;
use std::path::PathBuf;

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
        dirs.list(),
        [system.join("b.jpg"), system.join("release/a.png")]
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
