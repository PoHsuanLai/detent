//! A file that is there and cannot be used is never taken for a missing one (DETENT-01, -12):
//! it shows the defaults, is reported, and no save writes over it.

use crate::round_trip::{home, read_file, save_key, spec, word, write_file};
use detent_files::{FilesError, load};
use ds_settings::schema::KeyPath;
use std::os::unix::fs::{PermissionsExt, symlink};

/// What a refused save must leave behind: the file, byte for byte.
fn assert_refused_and_untouched(
    result: Result<(), FilesError>,
    path: &std::path::Path,
    bytes: &[u8],
) {
    assert!(
        matches!(result, Err(FilesError::Unreadable { .. })),
        "{result:?}"
    );
    assert_eq!(std::fs::read(path).expect("still there"), bytes);
}

#[test]
fn text_that_is_not_utf8_is_reported_and_not_written_over() {
    let (_scratch, home) = home("unreadable-latin1");
    let path = home.path_of(&spec().file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    let bytes = b"# caf\xe9\n[dock]\nposition = \"left\"\n";
    std::fs::write(&path, bytes).expect("write");

    let loaded = load(&home, &spec());
    assert_eq!(loaded.invalid.len(), 1, "the whole file is one invalid key");
    assert!(loaded.invalid[0].path.is_empty());
    assert_eq!(
        loaded.value.get(&KeyPath("dock.position".to_owned())),
        Some(&word("bottom")),
        "the defaults are shown"
    );
    assert_refused_and_untouched(save_key(&home, "dock.bounce", word("off")), &path, bytes);
}

#[test]
fn a_file_the_person_cannot_read_is_reported_and_not_written_over() {
    let (_scratch, home) = home("unreadable-mode0");
    write_file(&home, "[dock]\nposition = \"left\"\n");
    let path = home.path_of(&spec().file);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0)).expect("chmod");
    if std::fs::read(&path).is_ok() {
        // Running as root: permissions do not stop the read, so there is nothing to refuse.
        return;
    }
    let loaded = load(&home, &spec());
    assert_eq!(loaded.invalid.len(), 1, "{:?}", loaded.invalid);
    let refused = save_key(&home, "dock.bounce", word("off"));
    assert!(
        matches!(refused, Err(FilesError::Unreadable { .. })),
        "{refused:?}"
    );
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");
    assert_eq!(read_file(&home), "[dock]\nposition = \"left\"\n");
}

#[test]
fn a_directory_in_the_files_place_is_refused() {
    let (_scratch, home) = home("unreadable-directory");
    let path = home.path_of(&spec().file);
    std::fs::create_dir_all(&path).expect("a directory where the file goes");
    assert_eq!(load(&home, &spec()).invalid.len(), 1);
    let refused = save_key(&home, "dock.bounce", word("off"));
    assert!(
        matches!(refused, Err(FilesError::Unreadable { .. })),
        "{refused:?}"
    );
    assert!(path.is_dir());
}

#[test]
fn a_file_far_larger_than_settings_are_is_refused_whole() {
    let (_scratch, home) = home("unreadable-huge");
    let path = home.path_of(&spec().file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    let bytes = vec![b'#'; 2 * 1024 * 1024];
    std::fs::write(&path, &bytes).expect("write");
    assert_eq!(load(&home, &spec()).invalid.len(), 1);
    assert_refused_and_untouched(save_key(&home, "dock.bounce", word("off")), &path, &bytes);
}

#[test]
fn a_link_to_a_device_is_not_read() {
    let (_scratch, home) = home("unreadable-device");
    let path = home.path_of(&spec().file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    // Endless if it were read; a regular-file check stops it before the first byte.
    symlink("/dev/zero", &path).expect("link");
    assert_eq!(load(&home, &spec()).invalid.len(), 1);
    let refused = save_key(&home, "dock.bounce", word("off"));
    assert!(
        matches!(refused, Err(FilesError::Unreadable { .. })),
        "{refused:?}"
    );
}

#[test]
fn the_refusal_is_in_plain_words() {
    let (_scratch, home) = home("unreadable-words");
    write_file(&home, "[dock\nbounce = ");
    let garbled = save_key(&home, "dock.bounce", word("off")).expect_err("refused");
    let text = garbled.to_string();
    assert!(text.contains("Fix it by hand"), "{text}");
    for word in ["TOML", "D-Bus"] {
        assert!(!text.contains(word), "{word} in {text}");
    }
}
