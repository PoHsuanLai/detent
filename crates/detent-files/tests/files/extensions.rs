//! The installed Extensions over a scratch data home and config home: what the Look menu offers,
//! what the page lists, the switch that writes `quire/extensions.toml`, and a stored look that is
//! no longer installed.

use crate::round_trip::{key, word};
use detent_files::{ConfigHome, DataHome, ExtensionHome, load};
use detent_model::{
    ConfigFile, ExtensionKind, ExtensionStanding, ExtensionTrouble, FileSpec, InstalledExtensions,
};
use detent_testkit::{
    Scratch, install_file_icons, install_look, install_torn_look, install_your_changes,
};
use ds_settings::schema::{FilePath, KeyKind};

const TMP: &str = env!("CARGO_TARGET_TMPDIR");
const PAPER: &str = "org.example.Paper";

fn world(name: &str) -> (Scratch, ExtensionHome) {
    let scratch = Scratch::under(TMP, name);
    let home = ExtensionHome::under(
        &ConfigHome(scratch.join("config")),
        &DataHome(scratch.join("data")),
        &[],
    );
    (scratch, home)
}

/// A group's rows: the id of each Extension and where it stands.
type Rows = Vec<(String, ExtensionStanding)>;

/// The ids on offer for a look, as the menu lists them, and the rows of the page by group.
fn shape(read: &InstalledExtensions) -> (Vec<String>, Vec<(ExtensionKind, Rows)>) {
    let offered = read.offers.of("look").words;
    let groups = read
        .list
        .groups
        .iter()
        .map(|group| {
            let rows = group
                .rows
                .iter()
                .map(|row| (row.id.clone(), row.standing.clone()))
                .collect();
            (group.kind, rows)
        })
        .collect();
    (offered, groups)
}

#[test]
fn a_look_is_offered_after_standard_and_a_torn_one_is_listed_apart_with_its_reason() {
    let (scratch, home) = world("extensions-read");
    install_look(&scratch.join("data"), PAPER, "Paper", "Warm paper");
    install_torn_look(&scratch.join("data"), "org.example.Torn", "Torn");
    let read = home.read();
    let (offered, groups) = shape(&read);
    assert_eq!(offered, ["standard", PAPER]);
    assert_eq!(read.offers.of("look").labels.of(PAPER), Some("Paper"));
    assert_eq!(
        read.offers.of("look").labels.of("standard"),
        Some("Standard")
    );
    assert_eq!(
        groups,
        [
            (
                ExtensionKind::Looks,
                vec![(PAPER.to_owned(), ExtensionStanding::Usable { on: true })]
            ),
            (
                ExtensionKind::Unusable,
                vec![(
                    "org.example.Torn".to_owned(),
                    ExtensionStanding::Unusable(ExtensionTrouble::Reported(
                        "Some files in this Extension could not be read".to_owned()
                    ))
                )]
            ),
        ]
    );
    assert_eq!(
        read.list.groups[0].rows[0].summary.as_deref(),
        Some("Warm paper")
    );
}

#[test]
fn turning_one_off_writes_the_file_keeps_its_other_lines_and_turning_it_on_removes_the_row() {
    let (scratch, home) = world("extensions-power");
    install_look(&scratch.join("data"), PAPER, "Paper", "Warm paper");
    let file = scratch.join("config/quire/extensions.toml");
    std::fs::create_dir_all(file.parent().expect("a file has a parent")).expect("mkdir");
    std::fs::write(&file, "# mine\nversion = 1\n").expect("write");

    home.set_power(PAPER, false).expect("written");
    let text = std::fs::read_to_string(&file).expect("the file");
    assert!(text.starts_with("# mine\nversion = 1\n"), "{text}");
    assert!(text.contains(PAPER), "{text}");
    let (offered, groups) = shape(&home.read());
    assert_eq!(offered, ["standard"], "off leaves the menu");
    assert_eq!(
        groups,
        [(
            ExtensionKind::Looks,
            vec![(PAPER.to_owned(), ExtensionStanding::Usable { on: false })]
        )],
        "off stays listed, with its switch"
    );

    home.set_power(PAPER, false).expect("again");
    let again = std::fs::read_to_string(&file).expect("the file");
    assert_eq!(
        again.matches(PAPER).count(),
        1,
        "an id is listed once: {again}"
    );

    home.set_power(PAPER, true).expect("written");
    let (offered, _) = shape(&home.read());
    assert_eq!(offered, ["standard", PAPER]);
}

#[test]
fn a_file_that_cannot_be_understood_turns_nothing_off_and_is_not_written_over() {
    let (scratch, home) = world("extensions-garbled");
    install_look(&scratch.join("data"), PAPER, "Paper", "Warm paper");
    let file = scratch.join("config/quire/extensions.toml");
    std::fs::create_dir_all(file.parent().expect("a file has a parent")).expect("mkdir");
    std::fs::write(&file, "[extensions\noff = ").expect("write");
    assert_eq!(shape(&home.read()).0, ["standard", PAPER]);
    assert!(home.set_power(PAPER, false).is_err());
    assert_eq!(
        std::fs::read_to_string(&file).expect("the file"),
        "[extensions\noff = "
    );
}

#[test]
fn a_stored_look_nothing_installed_has_is_kept_on_load() {
    let scratch = Scratch::under(TMP, "extensions-stale-look");
    let file = ConfigFile::parse(&FilePath("quire/appearance.toml".to_owned())).expect("file");
    let spec = FileSpec {
        file,
        foreign: Default::default(),
        keys: vec![key(
            "appearance.look",
            KeyKind::Provided {
                capability: "look".to_owned(),
            },
            word("standard"),
        )],
    };
    std::fs::create_dir_all(scratch.join("config/quire")).expect("mkdir");
    std::fs::write(
        scratch.join("config/quire/appearance.toml"),
        "[appearance]\nlook = \"org.example.Gone\"\n",
    )
    .expect("write");
    let loaded = load(&ConfigHome(scratch.join("config")), &spec);
    let path = &spec.keys[0].path;
    assert_eq!(loaded.value.get(path), Some(&word("org.example.Gone")));
    assert_eq!(
        loaded.value.as_written(path),
        None,
        "a choice, not a mistake"
    );
    assert!(loaded.invalid.is_empty(), "{:?}", loaded.invalid);
}

#[test]
fn a_file_icons_set_is_offered_after_standard_and_listed_under_file_icons() {
    let (scratch, home) = world("extensions-file-icons");
    install_file_icons(
        &scratch.join("data"),
        "org.example.Dots",
        "Dots",
        "Round dots",
    );
    let read = home.read();
    let offer = read.offers.of("file_icons");
    assert_eq!(offer.words, ["standard", "org.example.Dots"]);
    assert_eq!(offer.labels.of("org.example.Dots"), Some("Dots"));
    let kinds: Vec<ExtensionKind> = read.list.groups.iter().map(|g| g.kind).collect();
    assert_eq!(kinds, [ExtensionKind::FileIcons]);
}

#[test]
fn your_changes_is_never_offered_and_has_a_switch_only_while_it_holds_nothing() {
    for holds in [false, true] {
        let (scratch, home) = world("extensions-your-changes");
        install_file_icons(
            &scratch.join("data"),
            "org.example.Dots",
            "Dots",
            "Round dots",
        );
        install_your_changes(&scratch.join("data"), holds);
        let read = home.read();
        assert_eq!(
            read.offers.of("file_icons").words,
            ["standard", "org.example.Dots"],
            "the person's changes are on top of the choice, never a choice"
        );
        let mine = read
            .list
            .groups
            .iter()
            .flat_map(|group| &group.rows)
            .find(|row| row.id == "user.YourChanges")
            .expect("the page lists it");
        assert_eq!(mine.holds_changes, holds);
        assert_eq!(mine.power().is_some(), !holds);
    }
}
