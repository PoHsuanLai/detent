//! Reading leniently and writing atomically, on a scratch config home.

use detent_files::{ConfigHome, FilesError, load, save};
use detent_model::{ConfigFile, FileSpec};
use detent_testkit::Scratch;
use ds_settings::schema::{
    Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section,
};
use toml::Value;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

pub(crate) fn key(path: &str, kind: KeyKind, default: Value) -> KeySpec {
    KeySpec {
        ends: None,
        control: None,
        path: KeyPath(path.to_owned()),
        kind,
        default,
        label: Label(path.to_owned()),
        help: Help(String::new()),
        page: Page::Dock,
        section: Section("S".to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        agent: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
    }
}

pub(crate) fn word(text: &str) -> Value {
    Value::String(text.to_owned())
}

pub(crate) fn spec() -> FileSpec {
    let file = ConfigFile::parse(&FilePath("acme/settings.toml".to_owned())).expect("file");
    FileSpec {
        file,
        foreign: Default::default(),
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

pub(crate) fn home(name: &str) -> (Scratch, ConfigHome) {
    let scratch = Scratch::under(TMP, name);
    let home = ConfigHome(scratch.path().to_path_buf());
    (scratch, home)
}

pub(crate) fn write_file(home: &ConfigHome, text: &str) {
    let path = home.path_of(&spec().file);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, text).expect("write");
}

pub(crate) fn read_file(home: &ConfigHome) -> String {
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
        "out of range: the control shows its default, the file's number stays (as_written)"
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

pub(crate) fn save_key(home: &ConfigHome, path: &str, value: Value) -> Result<(), FilesError> {
    save(home, &spec().file, &KeyPath(path.to_owned()), &value)
}

#[test]
fn a_value_no_control_holds_is_kept_as_written_and_reported() {
    let (_scratch, home) = home("files-as-written");
    write_file(
        &home,
        "[dock]\nbounce = { kind = \"keys\", v = \"alt+left\" }\nsize_px = 999\n",
    );
    let loaded = load(&home, &spec());
    let held = |path: &str| loaded.value.as_written(&KeyPath(path.to_owned())).cloned();
    assert_eq!(held("dock.size_px"), Some(Value::Integer(999)));
    assert!(matches!(held("dock.bounce"), Some(Value::Table(_))));
    assert_eq!(
        held("dock.position"),
        None,
        "a representable key is not held aside"
    );
    assert_eq!(
        loaded.unknown,
        Vec::new(),
        "the table value is the key's own"
    );
}

#[test]
fn a_write_sets_one_key_and_keeps_every_other_byte() {
    let (_scratch, home) = home("files-round-trip");
    let before = "# my dock\nversion = 1\n[dock]\nposition = \"right\"  # left handed\n\
        puppy = true\nbounce = { kind = \"keys\", v = \"alt+left\" }\nsize_px = 999\n\
        [mystery]\nx = 1\n";
    write_file(&home, before);
    save_key(&home, "dock.position", word("left")).expect("save");

    let want = before.replace("\"right\"", "\"left\"");
    assert_eq!(read_file(&home), want, "only the edited value changed");
    let dir = home.path_of(&spec().file);
    // The sidecar `.<name>.lock` is the update lock every writer of the file shares; it stays.
    let mut lock_name = std::ffi::OsString::from(".");
    lock_name.push(dir.file_name().expect("name"));
    lock_name.push(".lock");
    let leftovers = std::fs::read_dir(dir.parent().expect("parent"))
        .expect("dir")
        .filter_map(Result::ok)
        .filter(|e| e.path() != dir && e.file_name() != lock_name)
        .count();
    assert_eq!(leftovers, 0, "the temporary file was renamed away");
}

#[test]
fn a_write_to_a_missing_file_holds_only_that_key() {
    let (_scratch, home) = home("files-missing-write");
    save_key(&home, "dock.bounce", word("off")).expect("save");
    assert_eq!(read_file(&home), "[dock]\nbounce = \"off\"\n");
}

#[test]
fn a_write_into_a_table_the_file_lacks_makes_it() {
    let (_scratch, home) = home("files-new-table");
    write_file(&home, "[scroll]\nspeed = 1.5\n");
    save_key(&home, "dock.size_px", Value::Integer(64)).expect("save");
    let text = read_file(&home);
    assert!(text.starts_with("[scroll]\nspeed = 1.5\n"), "{text}");
    assert!(text.contains("[dock]\nsize_px = 64"), "{text}");
}

#[test]
fn a_fractional_number_stays_a_float() {
    let (_scratch, home) = home("files-float");
    save_key(&home, "scroll.speed", Value::Float(2.0)).expect("save");
    assert!(
        read_file(&home).contains("speed = 2.0"),
        "{}",
        read_file(&home)
    );
    let loaded = load(&home, &spec());
    assert_eq!(
        loaded.value.get(&KeyPath("scroll.speed".to_owned())),
        Some(&Value::Float(2.0))
    );
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
    let refused = save_key(&home, "dock.bounce", word("off"));
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
fn a_files_own_version_is_neither_reported_nor_touched() {
    let (_scratch, home) = home("files-version");
    write_file(&home, "version = 1\n[dock]\nposition = \"left\"\n");
    assert_eq!(load(&home, &spec()).unknown, Vec::new());
    save_key(&home, "dock.bounce", word("off")).expect("save");
    assert!(
        read_file(&home).starts_with("version = 1\n"),
        "{}",
        read_file(&home)
    );
}
