//! The installed apps, read from desktop entries in a scratch folder.

use detent_files::scan_programs;
use detent_model::{ProgramSubject, SubjectKind};
use detent_testkit::Scratch;
use std::path::{Path, PathBuf};

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

fn entry(dir: &Path, file: &str, text: &str) {
    std::fs::create_dir_all(dir).expect("dir");
    std::fs::write(dir.join(file), text).expect("write");
}

#[test]
fn entries_are_read_in_the_order_of_their_folders_and_name_the_program_they_start() {
    let scratch = Scratch::under(TMP, "programs-scan");
    let (first, second) = (scratch.join("first"), scratch.join("second"));
    entry(
        &first,
        "grim.desktop",
        "[Desktop Entry]\nType=Application\nName=Grim\nExec=/usr/bin/grim %f\nIcon=/icons/grim.svg\n",
    );
    entry(
        &first,
        "loupe.desktop",
        "[Desktop Entry]\nType=Application\nName=Image Viewer\nExec=flatpak run org.gnome.Loupe\nX-Flatpak=org.gnome.Loupe\nIcon=no-such-icon-in-any-theme-detent\n",
    );
    entry(
        &second,
        "link.desktop",
        "[Desktop Entry]\nType=Link\nName=Not an app\nURL=https://example.org\n",
    );
    entry(&second, "broken.desktop", "this is not an entry");
    let book = scan_programs(&[first, second, scratch.join("missing")]);

    let ids: Vec<&str> = book.apps.iter().map(|app| app.id.as_str()).collect();
    assert_eq!(
        ids,
        ["grim", "loupe"],
        "a link and a broken file are not apps"
    );
    let look = book.look_of(&ProgramSubject("/usr/bin/grim".to_owned()));
    assert_eq!(look.name, "Grim");
    assert_eq!(look.icon, Some(PathBuf::from("/icons/grim.svg")));
    let loupe = book.look_of(&ProgramSubject("org.gnome.Loupe".to_owned()));
    assert_eq!(loupe.name, "Image Viewer");
    assert_eq!(loupe.icon, None, "an icon no theme has is no icon");
    assert_eq!(
        ProgramSubject("org.gnome.Loupe".to_owned()).kind(),
        SubjectKind::Sandboxed
    );
}

#[test]
fn a_folder_inside_applications_names_its_entries_and_a_link_back_up_is_walked_once() {
    let scratch = Scratch::under(TMP, "programs-nested");
    let applications = scratch.join("share/applications");
    let nested = applications.join("kde");
    entry(
        &nested,
        "okular.desktop",
        "[Desktop Entry]\nType=Application\nName=Okular\nName[de]=Okular (de)\nExec=okular %U\n",
    );
    entry(
        &applications,
        "big.desktop",
        &format!(
            "[Desktop Entry]\nType=Application\nName=Big\nComment={}\n",
            "x".repeat(300 * 1024)
        ),
    );
    std::os::unix::fs::symlink(&applications, nested.join("back")).expect("link");
    let book = scan_programs(&[applications]);

    let found: Vec<(&str, &str)> = book
        .apps
        .iter()
        .map(|app| (app.id.as_str(), app.name.as_str()))
        .collect();
    assert_eq!(
        found,
        [("kde-okular", "Okular")],
        "the plain name, once; an entry over the cap is not read"
    );
}
