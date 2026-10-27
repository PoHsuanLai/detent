//! What a program is called: the system entry that names it, else its file and folder, and a name
//! that cannot hide or reorder text.

use super::{LOUPE, subject};
use crate::origin::SchemaOrigin;
use crate::privacy::{DesktopApp, ProgramBook};

fn app(id: &str, name: &str, exec: Option<&str>, flatpak: Option<&str>) -> DesktopApp {
    DesktopApp {
        id: id.to_owned(),
        name: name.to_owned(),
        exec: exec.map(str::to_owned),
        try_exec: None,
        flatpak: flatpak.map(str::to_owned),
        icon: Some(std::path::PathBuf::from(format!("/icons/{id}.svg"))),
        origin: SchemaOrigin::System,
    }
}

fn planted(mut entry: DesktopApp) -> DesktopApp {
    entry.origin = SchemaOrigin::PersonWritable;
    entry
}

fn book_of(apps: Vec<DesktopApp>) -> ProgramBook {
    ProgramBook {
        apps,
        system_bins: vec![std::path::PathBuf::from("/usr/bin")],
    }
}

#[test]
fn a_program_is_named_by_the_system_entry_that_starts_it_else_by_its_file_and_folder() {
    let mut try_only = app("try", "Tried", None, None);
    try_only.try_exec = Some("/opt/try/bin/tried".to_owned());
    let book = book_of(vec![
        app("grim", "Grim", Some("/usr/bin/grim"), None),
        app("bare", "Bare", Some("bare"), None),
        try_only,
        app("loupe", "Image Viewer", Some("flatpak"), Some(LOUPE)),
        app("nameless", "", Some("/usr/bin/nameless"), None),
    ]);
    // `(case, subject, name, has the entry's icon, the folder shown)`.
    const CASES: &[(&str, &str, &str, bool, Option<&str>)] = &[
        ("the same path", "/usr/bin/grim", "Grim", true, None),
        (
            "a bare name is the image's file name in a system folder",
            "/usr/bin/bare",
            "Bare",
            true,
            None,
        ),
        (
            "a bare name does not name an image elsewhere",
            "/opt/x/bare",
            "bare",
            false,
            Some("/opt/x"),
        ),
        (
            "TryExec starts the image",
            "/opt/try/bin/tried",
            "Tried",
            true,
            None,
        ),
        (
            "a sandboxed app by its id",
            LOUPE,
            "Image Viewer",
            true,
            None,
        ),
        (
            "an entry of a sandboxed app starts no host program",
            "/usr/bin/flatpak",
            "flatpak",
            false,
            Some("/usr/bin"),
        ),
        (
            "no entry: the file name and its folder",
            "/usr/bin/unknown",
            "unknown",
            false,
            Some("/usr/bin"),
        ),
        (
            "no entry: the app's id",
            "org.example.Gone",
            "org.example.Gone",
            false,
            None,
        ),
        (
            "an entry without a name: the file name, the entry's icon",
            "/usr/bin/nameless",
            "nameless",
            true,
            Some("/usr/bin"),
        ),
        ("nothing to go by", "", "", false, None),
    ];
    for (case, who, name, icon, place) in CASES {
        let look = book.look_of(&subject(who));
        assert_eq!(look.name.as_deref().unwrap_or_default(), *name, "{case}");
        assert_eq!(look.icon.is_some(), *icon, "{case}");
        assert_eq!(look.place.as_deref(), *place, "{case}");
    }
    let long = book_of(vec![app(
        "long",
        &"x".repeat(100),
        Some("/usr/bin/long"),
        None,
    )]);
    let name = long
        .look_of(&subject("/usr/bin/long"))
        .name
        .unwrap_or_default();
    assert_eq!(name.chars().count(), 48);
    assert!(name.ends_with('\u{2026}'));
}

#[test]
fn an_entry_a_program_planted_in_a_folder_the_person_can_write_names_nothing() {
    // `(case, the entry, the subject, the name shown)`: the entry says "Firefox"; the program is
    // not the real one, and says what it is.
    let cases = [
        (
            "the planted entry's launch line is the program's own path",
            planted(app("x", "Firefox", Some("/tmp/evil"), None)),
            "/tmp/evil",
            "evil",
        ),
        (
            "the planted entry's launch line is a bare name",
            planted(app("x", "Firefox", Some("evil"), None)),
            "/usr/bin/evil",
            "evil",
        ),
        (
            "the program is called firefox, in a folder the person can write",
            app("firefox", "Firefox", Some("firefox"), None),
            "/home/ada/bin/firefox",
            "firefox",
        ),
        (
            "a planted entry for a sandboxed app's id",
            planted(app("org.evil.App", "Firefox", None, Some("org.evil.App"))),
            "org.evil.App",
            "org.evil.App",
        ),
    ];
    for (case, entry, who, want) in cases {
        let look = book_of(vec![entry]).look_of(&subject(who));
        assert_eq!(look.name.as_deref(), Some(want), "{case}");
        assert_eq!(look.icon, None, "{case}: no borrowed icon");
    }
    let shown = book_of(vec![planted(app("x", "Firefox", Some("/tmp/evil"), None))])
        .look_of(&subject("/tmp/evil"));
    assert_eq!(shown.place.as_deref(), Some("/tmp"));

    // A planted entry ahead of the system's own does not take its place.
    let both = book_of(vec![
        planted(app("x", "Firefox", Some("/usr/bin/firefox"), None)),
        app("firefox", "Mozilla Firefox", Some("/usr/bin/firefox"), None),
    ]);
    assert_eq!(
        both.look_of(&subject("/usr/bin/firefox")).name.as_deref(),
        Some("Mozilla Firefox")
    );
}

#[test]
fn a_name_that_hides_or_reorders_text_is_drawn_plainly() {
    let look = book_of(Vec::new()).look_of(&subject("/tmp/ev\u{202E}il\nname"));
    assert_eq!(look.name.as_deref(), Some("ev\u{FFFD}il\u{FFFD}name"));
    let deep = format!("/{}/prog", "d".repeat(100));
    let place = book_of(Vec::new()).look_of(&subject(&deep)).place;
    let place = place.unwrap_or_default();
    assert_eq!(place.chars().count(), 60);
    assert!(place.starts_with('\u{2026}') && place.ends_with('d'));
}
