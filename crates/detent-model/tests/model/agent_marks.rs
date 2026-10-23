//! Which keys an agent may set: the ones a schema marks `agent = "settable"`, and no other.

use detent_model::{AgentSetting, Catalogue, OriginRule, SchemaOrigin, discover, discover_under};
use detent_testkit::{Scratch, fixture_dir};
use ds_settings::schema::KeyPath;
use std::os::unix::fs::PermissionsExt;
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

/// The catalogue of `dirs`, of which `system` are the folders only an administrator can write.
fn catalogue(system: &[&PathBuf], dirs: &[PathBuf]) -> Catalogue {
    let system: Vec<PathBuf> = system.iter().map(|dir| (*dir).clone()).collect();
    let found = discover_under(dirs, &OriginRule::declared(&system));
    Catalogue::build(&found.programs)
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
    let dir = schema_dir(&scratch, "d", &text);
    let found = catalogue(&[&dir], std::slice::from_ref(&dir));
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
    let found = catalogue(&[&dir], std::slice::from_ref(&dir));
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
    let found = catalogue(&[&first, &second], &[first.clone(), second.clone()]);
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
    let found = catalogue(&[&dir], std::slice::from_ref(&dir));
    assert_eq!(found.entry(&KeyPath(MAGNIFICATION.to_owned())), None);
}

#[test]
fn a_user_schema_cannot_lend_its_marks_to_an_installed_one() {
    let scratch = Scratch::under(TMP, "marks-user-dir");
    let plain = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let system = schema_dir(&scratch, "system", &plain);
    let user = schema_dir(&scratch, "user", &sill_marking(MAGNIFICATION));
    // `ds_settings::schema::data_dirs` lists the system's directories first.
    let found = catalogue(&[&system], &[system.clone(), user]);
    assert_eq!(setting_of(&found, MAGNIFICATION), AgentSetting::HandsOff);
}

#[test]
fn a_stale_schema_that_marks_keys_lends_nothing_to_the_valid_one() {
    let scratch = Scratch::under(TMP, "marks-stale");
    let plain = std::fs::read_to_string(fixture_dir().join("sill.settings.toml")).expect("fixture");
    let system = schema_dir(&scratch, "system", &plain);
    let stale = sill_marking(MAGNIFICATION).replacen("version = 1", "version = 99", 1);
    let user = schema_dir(&scratch, "user", &stale);
    let found = discover_under(
        &[system.clone(), user],
        &OriginRule::declared(std::slice::from_ref(&system)),
    );
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
    let text = sill_marking(MAGNIFICATION).replacen(
        &format!("path = \"{MAGNIFICATION}\""),
        "path = \"ai.magnification\"",
        1,
    );
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
    let found = catalogue(&[&dir], std::slice::from_ref(&dir));
    assert_eq!(setting_of(&found, MAGNIFICATION), AgentSetting::HandsOff);
}

#[test]
fn a_schema_from_a_folder_the_person_can_write_is_shown_but_marks_nothing() {
    let scratch = Scratch::under(TMP, "marks-person-writable");
    let dir = schema_dir(&scratch, "home", &sill_marking(MAGNIFICATION));
    // The folder is not declared a system one, so it is judged as the person's.
    let found = catalogue(&[], std::slice::from_ref(&dir));
    let as_system = catalogue(&[&dir], std::slice::from_ref(&dir));
    assert_eq!(
        found.entries().count(),
        as_system.entries().count(),
        "its keys are all still shown"
    );
    assert_eq!(setting_of(&found, MAGNIFICATION), AgentSetting::HandsOff);
    assert!(
        found
            .entries()
            .all(|(_, entry)| entry.agent() == AgentSetting::HandsOff)
    );
}

#[test]
fn ownership_decides_a_folder_when_nothing_is_declared() {
    let scratch = Scratch::under(TMP, "marks-ownership");
    let open = schema_dir(&scratch, "open", &sill_marking(MAGNIFICATION));
    std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o777)).expect("chmod");
    let found = discover(std::slice::from_ref(&open));
    assert_eq!(found.programs.len(), 1, "the schema is still a program");
    let catalogue = Catalogue::build(&found.programs);
    assert_eq!(
        setting_of(&catalogue, MAGNIFICATION),
        AgentSetting::HandsOff,
        "a writable folder, whoever owns it, lends no marks"
    );
}

#[test]
fn a_folder_is_system_only_when_root_owns_it_and_every_folder_above_it() {
    let scratch = Scratch::under(TMP, "marks-origins");
    let own = scratch.join("own");
    std::fs::create_dir_all(&own).expect("dir");
    // `/usr` is root's and closed to everyone else on any machine this runs on.
    assert_eq!(
        SchemaOrigin::of_dir(std::path::Path::new("/usr")),
        SchemaOrigin::System
    );
    assert_eq!(
        SchemaOrigin::of_dir(&scratch.join("missing")),
        SchemaOrigin::PersonWritable,
        "a folder that cannot be read is the person's"
    );
    std::fs::set_permissions(&own, std::fs::Permissions::from_mode(0o777)).expect("chmod");
    assert_eq!(SchemaOrigin::of_dir(&own), SchemaOrigin::PersonWritable);
}

#[test]
fn quires_schema_marks_theme_accent_look_file_icons_and_use_my_styles_and_use_my_styles_is_a_switch()
 {
    use detent_model::{Control, PageSlug, TogglePair};
    let scratch = Scratch::under(TMP, "marks-quire-appearance");
    let dir = scratch.join("d");
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::copy(
        fixture_dir().join("quire.settings.toml"),
        dir.join("quire.settings.toml"),
    )
    .expect("copy");
    let found = catalogue(&[&dir], std::slice::from_ref(&dir));
    let settable: Vec<&str> = found
        .entries()
        .filter(|(_, e)| e.agent() == AgentSetting::Settable)
        .map(|(_, e)| e.spec.path.0.as_str())
        .collect();
    assert_eq!(
        settable,
        [
            "appearance.theme",
            "appearance.accent",
            "appearance.look",
            "appearance.custom_styles",
            // In its own section (Files), which follows the Appearance section's keys.
            "appearance.file_icons"
        ]
    );
    let entry = found
        .entry(&KeyPath("appearance.custom_styles".to_owned()))
        .expect("the key is in the catalogue");
    assert_eq!(
        entry.control,
        Control::Toggle(TogglePair {
            on: "use".to_owned(),
            off: "off".to_owned()
        })
    );
    let page = found
        .page(&PageSlug("appearance".to_owned()))
        .expect("page");
    assert!(page.sections.iter().any(|s| s.keys.contains(entry)));
}
