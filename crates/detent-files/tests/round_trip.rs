//! Reading leniently and writing atomically, on a scratch config home.

use detent_files::{ConfigHome, FilesError, load, save};
use detent_model::{ConfigFile, FileSpec};
use detent_testkit::Scratch;
use ds_settings::schema::{
    Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section,
};
use toml::Value;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

fn key(path: &str, kind: KeyKind, default: Value) -> KeySpec {
    KeySpec {
        path: KeyPath(path.to_owned()),
        kind,
        default,
        label: Label(path.to_owned()),
        help: Help(String::new()),
        page: Page::Dock,
        section: Section("S".to_owned()),
        exposure: Exposure::Basic,
    }
}

fn word(text: &str) -> Value {
    Value::String(text.to_owned())
}

fn spec() -> FileSpec {
    let file = ConfigFile::parse(&FilePath("acme/settings.toml".to_owned())).expect("file");
    FileSpec {
        file,
        keys: vec![
            key(
                "dock.bounce",
                KeyKind::Toggle {
                    variants: ["on".to_owned(), "off".to_owned()],
                },
                word("on"),
            ),
            key(
                "dock.position",
                KeyKind::Segmented {
                    variants: vec!["bottom".to_owned(), "left".to_owned(), "right".to_owned()],
                },
                word("bottom"),
            ),
            key(
                "dock.size_px",
                KeyKind::Bounded {
                    min: 16,
                    max: 128,
                    unit: Some("px".to_owned()),
                },
                Value::Integer(48),
            ),
            key(
                "scroll.speed",
                KeyKind::Bounded {
                    min: 0,
                    max: 4,
                    unit: None,
                },
                Value::Float(1.5),
            ),
        ],
    }
}

fn home(name: &str) -> (Scratch, ConfigHome) {
    let scratch = Scratch::under(TMP, name);
    let home = ConfigHome(scratch.path().to_path_buf());
    (scratch, home)
}

fn write_file(home: &ConfigHome, text: &str) {
    let path = home.path_of(&spec().file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, text).expect("write");
}

fn read_file(home: &ConfigHome) -> String {
    std::fs::read_to_string(home.path_of(&spec().file)).expect("read")
}

#[test]
fn a_missing_file_is_the_defaults_and_reports_nothing() {
    let (_scratch, home) = home("files-missing");
    let loaded = load(&home, &spec());
    assert_eq!(loaded.unknown, Vec::new());
    assert_eq!(loaded.invalid, Vec::new());
    for key in spec().keys {
        assert_eq!(
            loaded.value.get(&key.path),
            Some(&key.default),
            "{}",
            key.path.0
        );
    }
}

#[test]
fn a_bad_value_costs_only_its_own_key() {
    let (_scratch, home) = home("files-lenient");
    write_file(
        &home,
        "[dock]\nbounce = \"bounceyy\"\nposition = \"left\"\nsize_px = 999\n\
         puppy = true\n[mystery]\nx = 1\ny = 2\n",
    );
    let loaded = load(&home, &spec());
    let got = |path: &str| loaded.value.get(&KeyPath(path.to_owned())).cloned();
    assert_eq!(
        got("dock.bounce"),
        Some(word("on")),
        "unknown word: default"
    );
    assert_eq!(
        got("dock.position"),
        Some(word("left")),
        "sibling still read"
    );
    assert_eq!(
        got("dock.size_px"),
        Some(Value::Integer(48)),
        "out of range: default"
    );
    let invalid: Vec<&str> = loaded.invalid.iter().map(|k| k.path.as_str()).collect();
    assert_eq!(invalid, ["dock.bounce", "dock.size_px"]);
    let unknown: Vec<&str> = loaded.unknown.iter().map(|k| k.0.as_str()).collect();
    assert_eq!(
        unknown,
        ["dock.puppy", "mystery"],
        "a whole unknown table is one entry"
    );
}

#[test]
fn a_write_round_trips_and_drops_what_no_schema_describes() {
    let (_scratch, home) = home("files-round-trip");
    write_file(&home, "[dock]\nposition = \"right\"\npuppy = true\n");
    let loaded = load(&home, &spec());
    let edited = loaded
        .value
        .with(&KeyPath("dock.bounce".to_owned()), word("off"));
    save(&home, &spec(), &edited).expect("save");

    let text = read_file(&home);
    assert!(
        !text.contains("puppy"),
        "unknown keys are not written back:\n{text}"
    );
    assert!(
        text.contains("bounce = \"off\""),
        "the edit is written:\n{text}"
    );

    let again = load(&home, &spec());
    assert_eq!(again.unknown, Vec::new());
    assert_eq!(again.value, edited);
    let dir = home.path_of(&spec().file);
    let leftovers = std::fs::read_dir(dir.parent().expect("parent"))
        .expect("dir")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "part"))
        .count();
    assert_eq!(leftovers, 0, "the temporary file was renamed away");
}

#[test]
fn a_fractional_number_stays_a_float() {
    let (_scratch, home) = home("files-float");
    let loaded = load(&home, &spec());
    let edited = loaded
        .value
        .with(&KeyPath("scroll.speed".to_owned()), Value::Float(2.0));
    save(&home, &spec(), &edited).expect("save");
    assert!(
        read_file(&home).contains("speed = 2.0"),
        "{}",
        read_file(&home)
    );
    assert_eq!(load(&home, &spec()).value, edited);
}

#[test]
fn a_file_that_is_not_toml_is_left_alone() {
    let (_scratch, home) = home("files-garbled");
    write_file(&home, "[dock\nbounce = ");
    let loaded = load(&home, &spec());
    assert_eq!(
        loaded.invalid.len(),
        1,
        "the whole file is the one invalid key"
    );
    let refused = save(&home, &spec(), &loaded.value);
    assert!(
        matches!(refused, Err(FilesError::Garbled { .. })),
        "{refused:?}"
    );
    assert_eq!(
        read_file(&home),
        "[dock\nbounce = ",
        "what the person typed is kept"
    );
}

#[test]
fn a_files_own_version_is_carried_through() {
    let (_scratch, home) = home("files-version");
    write_file(&home, "version = 1\n[dock]\nposition = \"left\"\n");
    let loaded = load(&home, &spec());
    assert_eq!(loaded.unknown, Vec::new(), "version is the file's own key");
    save(&home, &spec(), &loaded.value).expect("save");
    assert!(
        read_file(&home).starts_with("version = 1"),
        "{}",
        read_file(&home)
    );
}
