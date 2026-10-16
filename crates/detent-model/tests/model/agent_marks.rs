//! Which keys an agent may set: the ones a schema marks `agent = "settable"`, and no other.

use detent_model::{AgentSetting, Catalogue, discover};
use detent_testkit::{Scratch, fixture_dir};
use ds_settings::schema::KeyPath;
use std::path::PathBuf;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");
const MAGNIFICATION: &str = "dock.magnification";

/// sill's real schema with `agent = "settable"` written under the key `path`.
fn sill_marking(path: &str) -> String {
    let text = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let line = format!("path = \"{path}\"");
    assert!(text.contains(&line), "the fixture has {path}");
    text.replacen(&line, &format!("{line}\nagent = \"settable\""), 1)
}

fn schema_dir(scratch: &Scratch, name: &str, text: &str) -> PathBuf {
    let dir = scratch.join(name);
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(dir.join("sill.settings.toml"), text).expect("write");
    dir
}

fn catalogue(dirs: &[PathBuf]) -> Catalogue {
    Catalogue::build(&discover(dirs).programs)
}

fn setting_of(catalogue: &Catalogue, path: &str) -> AgentSetting {
    catalogue
        .entry(&KeyPath(path.to_owned()))
        .unwrap_or_else(|| panic!("{path} is in the catalogue"))
        .agent()
}

#[test]
fn a_schema_without_marks_leaves_every_key_to_the_person() {
    let scratch = Scratch::under(TMP, "marks-none");
    let text = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let found = catalogue(&[schema_dir(&scratch, "d", &text)]);
    assert!(
        found
            .entries()
            .all(|(_, entry)| entry.agent() == AgentSetting::HandsOff)
    );
}

#[test]
fn only_the_marked_key_is_settable() {
    let scratch = Scratch::under(TMP, "marks-one");
    let dir = schema_dir(&scratch, "d", &sill_marking(MAGNIFICATION));
    let found = catalogue(&[dir]);
    assert_eq!(setting_of(&found, MAGNIFICATION), AgentSetting::Settable);
    let settable = found
        .entries()
        .filter(|(_, e)| e.agent() == AgentSetting::Settable)
        .count();
    assert_eq!(settable, 1, "no other key is marked");
}

#[test]
fn the_marks_come_from_the_schema_that_supplies_the_program() {
    let scratch = Scratch::under(TMP, "marks-shadow");
    let plain = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let first = schema_dir(&scratch, "home", &plain);
    let second = schema_dir(&scratch, "system", &sill_marking(MAGNIFICATION));
    let found = catalogue(&[first, second]);
    assert_eq!(
        setting_of(&found, MAGNIFICATION),
        AgentSetting::HandsOff,
        "the earlier directory shadows the later one, marks included"
    );
}

#[test]
fn a_path_two_programs_share_names_no_one_key() {
    let scratch = Scratch::under(TMP, "marks-ambiguous");
    let dir = scratch.join("d");
    std::fs::create_dir_all(&dir).expect("dir");
    let text = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    std::fs::write(dir.join("sill.settings.toml"), &text).expect("write");
    std::fs::write(
        dir.join("twin.settings.toml"),
        text.replace("app = \"sill\"", "app = \"twin\"")
            .replace("sill/settings.toml", "twin/settings.toml"),
    )
    .expect("write");
    let found = catalogue(&[dir]);
    assert_eq!(found.entry(&KeyPath(MAGNIFICATION.to_owned())), None);
}

#[test]
fn a_user_schema_cannot_lend_its_marks_to_an_installed_one() {
    let scratch = Scratch::under(TMP, "marks-user-dir");
    let plain = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let system = schema_dir(&scratch, "system", &plain);
    let user = schema_dir(&scratch, "user", &sill_marking(MAGNIFICATION));
    // `ds_settings::schema::data_dirs` lists the system's directories first.
    let found = catalogue(&[system, user]);
    assert_eq!(setting_of(&found, MAGNIFICATION), AgentSetting::HandsOff);
}

#[test]
fn a_stale_schema_that_marks_keys_lends_nothing_to_the_valid_one() {
    let scratch = Scratch::under(TMP, "marks-stale");
    let plain = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let system = schema_dir(&scratch, "system", &plain);
    let stale = sill_marking(MAGNIFICATION).replacen("version = 1", "version = 99", 1);
    let user = schema_dir(&scratch, "user", &stale);
    let found = discover(&[system, user]);
    assert_eq!(found.programs.len(), 1);
    let catalogue = Catalogue::build(&found.programs);
    assert_eq!(
        setting_of(&catalogue, MAGNIFICATION),
        AgentSetting::HandsOff
    );
}

#[test]
fn a_schema_that_marks_a_never_settable_key_is_not_a_program() {
    let scratch = Scratch::under(TMP, "marks-never");
    let text = sill_marking(MAGNIFICATION).replacen(MAGNIFICATION, "ai.magnification", 1);
    let found = discover(&[schema_dir(&scratch, "d", &text)]);
    assert_eq!(found.programs, Vec::new());
}

#[test]
fn a_file_that_is_not_a_schema_marks_nothing_even_if_it_says_settable() {
    let scratch = Scratch::under(TMP, "marks-side-file");
    let plain = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let dir = schema_dir(&scratch, "d", &plain);
    // Not named for its program: the side reader that once took marks from any such file is gone.
    std::fs::write(dir.join("evil.settings.toml"), sill_marking(MAGNIFICATION)).expect("write");
    let found = catalogue(&[dir]);
    assert_eq!(setting_of(&found, MAGNIFICATION), AgentSetting::HandsOff);
}
