//! How a save lands (DETENT-03, -04, -15): the file's mode and a link to it survive, writers
//! take turns, and several keys arrive in one rename.

use crate::round_trip::{home, read_file, save_key, spec, word, write_file};
use detent_files::{ConfigHome, load_places, save_places};
use docket_settings::{ModelChoice, OwnComputer, PlaceName, Places, Toggle};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::Arc;

#[test]
fn a_private_file_stays_private() {
    let (_scratch, home) = home("writes-mode");
    write_file(&home, "[dock]\nposition = \"left\"\n");
    let path = home.path_of(&spec().file);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("chmod");
    save_key(&home, "dock.bounce", word("off")).expect("save");
    let mode = std::fs::metadata(&path).expect("meta").permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn a_linked_settings_file_stays_a_link_and_its_target_changes() {
    let (scratch, home) = home("writes-link");
    let path = home.path_of(&spec().file);
    let target = scratch.join("dotfiles/settings.toml");
    std::fs::create_dir_all(target.parent().expect("parent")).expect("mkdir");
    std::fs::write(&target, "[dock]\nposition = \"left\"\n").expect("write");
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    symlink(&target, &path).expect("link");

    save_key(&home, "dock.bounce", word("off")).expect("save");

    assert!(
        std::fs::symlink_metadata(&path)
            .expect("meta")
            .file_type()
            .is_symlink(),
        "the link was replaced by a copy"
    );
    let text = std::fs::read_to_string(&target).expect("target");
    assert!(text.contains("bounce = \"off\""), "{text}");
    assert!(text.contains("position = \"left\""), "{text}");
}

#[test]
fn saves_from_two_threads_both_land() {
    let (_scratch, home) = home("writes-turns");
    let home = Arc::new(home);
    let keys = [
        ("dock.bounce", word("off")),
        ("dock.position", word("right")),
        ("dock.size_px", toml::Value::Integer(64)),
        ("scroll.speed", toml::Value::Float(2.5)),
    ];
    let threads: Vec<_> = keys
        .iter()
        .cycle()
        .take(48)
        .cloned()
        .map(|(path, value)| {
            let home = Arc::clone(&home);
            std::thread::spawn(move || save_key(&home, path, value).expect("save"))
        })
        .collect();
    for thread in threads {
        thread.join().expect("a thread");
    }
    let text = read_file(&home);
    for want in [
        "bounce = \"off\"",
        "position = \"right\"",
        "size_px = 64",
        "speed = 2.5",
    ] {
        assert!(text.contains(want), "{want} missing from\n{text}");
    }
}

fn places() -> Places {
    Places {
        this_computer: Some(ModelChoice("small".to_owned())),
        own: vec![OwnComputer {
            name: PlaceName("lab server".to_owned()),
            allowed: Toggle::On,
            model: None,
        }],
        cloud: Vec::new(),
    }
}

#[test]
fn the_places_are_refused_over_a_file_that_cannot_be_read() {
    let scratch = detent_testkit::Scratch::under(env!("CARGO_TARGET_TMPDIR"), "writes-places");
    let home = ConfigHome(scratch.path().to_path_buf());
    let file = scratch.join("docket/settings.toml");
    std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
    let bytes = b"# caf\xe9\nnote = \"kept\"\n";
    std::fs::write(&file, bytes).expect("seed");
    let refused = save_places(&home, &places());
    assert!(
        matches!(refused, Err(detent_files::FilesError::Unreadable { .. })),
        "{refused:?}"
    );
    assert_eq!(std::fs::read(&file).expect("still there"), bytes);
}

#[test]
fn the_places_land_together_beside_the_keys_they_do_not_own() {
    let scratch = detent_testkit::Scratch::under(env!("CARGO_TARGET_TMPDIR"), "writes-places-ok");
    let home = ConfigHome(scratch.path().to_path_buf());
    let file = scratch.join("docket/settings.toml");
    std::fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
    std::fs::write(&file, "note = \"kept\"\n").expect("seed");
    save_places(&home, &places()).expect("saved");
    assert_eq!(load_places(&home).places, places());
    let text = std::fs::read_to_string(&file).expect("read");
    assert!(text.contains("note = \"kept\""), "{text}");
}
