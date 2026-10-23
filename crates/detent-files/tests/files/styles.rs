//! The style files the person wrote, over a scratch settings folder.

use detent_files::{ConfigHome, DataHome, StyleSheet, published_classes, read_style_sheets};
use detent_model::StyleOwner;
use detent_testkit::Scratch;
use std::fs;

fn write(scratch: &Scratch, relative: &str, text: &str) {
    let path = scratch.join(relative);
    fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
    fs::write(path, text).expect("write");
}

fn sheet(owner: StyleOwner, css: &str) -> StyleSheet {
    StyleSheet {
        owner,
        css: css.to_owned(),
    }
}

#[test]
fn the_desktop_sheet_comes_first_then_each_app_with_one_in_name_order() {
    let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "style-sheets");
    write(&scratch, "sill/style.css", "b");
    write(&scratch, "quire/style.css", "a");
    write(&scratch, "files/style.css", "c");
    write(&scratch, "mailo/settings.toml", "not a sheet");
    write(&scratch, "loose.css", "not in a folder");
    let found = read_style_sheets(&ConfigHome(scratch.path().to_owned()));
    assert_eq!(
        found,
        [
            sheet(StyleOwner::Desktop, "a"),
            sheet(StyleOwner::App("files".to_owned()), "c"),
            sheet(StyleOwner::App("sill".to_owned()), "b"),
        ]
    );
}

#[test]
fn a_missing_folder_or_a_sheet_that_cannot_be_read_leaves_the_file_out() {
    let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "style-sheets-unreadable");
    assert!(read_style_sheets(&ConfigHome(scratch.join("nowhere"))).is_empty());
    // A folder where the sheet should be, and bytes that are not text.
    fs::create_dir_all(scratch.join("sill/style.css")).expect("mkdir");
    write(&scratch, "files/style.css", "ok");
    fs::create_dir_all(scratch.join("quire")).expect("mkdir");
    fs::write(scratch.join("quire/style.css"), b"\xff\xfe").expect("write");
    let found = read_style_sheets(&ConfigHome(scratch.path().to_owned()));
    assert_eq!(found, [sheet(StyleOwner::App("files".to_owned()), "ok")]);
}

#[test]
fn the_classes_apps_publish_are_read_from_the_system_folders_then_the_persons() {
    let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "published-classes");
    let parts = |app: &str, class: &str| format!("app = \"{app}\"\nclasses = [\"{class}\"]\n");
    write(
        &scratch,
        "system/quire/selectors/sill.toml",
        &parts("sill", "sill-bar"),
    );
    write(
        &scratch,
        "data/quire/selectors/files.toml",
        &parts("files", "files-row"),
    );
    let found = published_classes(&DataHome(scratch.join("data")), &[scratch.join("system")]);
    assert_eq!(
        found,
        [vec!["sill-bar".to_owned()], vec!["files-row".to_owned()]]
    );
    assert!(published_classes(&DataHome(scratch.join("nowhere")), &[]).is_empty());
}
