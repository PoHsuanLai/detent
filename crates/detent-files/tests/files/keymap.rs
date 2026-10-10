//! The keymap files over a scratch home: read leniently, the person's written whole through a
//! rename, and the stamp that tells a change.

use chordkit::StandardAction;
use detent_files::{ConfigHome, KeymapFiles};
use detent_model::{Origin, ShortcutsEffect, ShortcutsInput, ShortcutsState, shortcuts_step};
use detent_testkit::Scratch;

fn scratch(name: &str) -> Scratch {
    Scratch::under(env!("CARGO_TARGET_TMPDIR"), name)
}

fn files(scratch: &Scratch) -> KeymapFiles {
    KeymapFiles::under(&ConfigHome(scratch.join("config")))
        .with_machine_dir(&scratch.path().join("etc"))
}

#[test]
fn files_that_are_not_there_are_empty() {
    let dir = scratch("keymap-missing");
    let texts = files(&dir).read();
    assert_eq!(texts.person.text, "");
    assert_eq!(texts.person.trouble, None);
    assert_eq!(texts.machine.text, "");
}

#[test]
fn the_person_file_is_written_whole_in_keycaps_place_and_read_back() {
    let dir = scratch("keymap-save");
    let files = files(&dir);
    files
        .save_person("# mine\ncopy = Super+Alt+C\n")
        .expect("saved");
    let path = dir.join("config/keycap/keymap");
    assert_eq!(
        std::fs::read_to_string(&path).expect("the file"),
        "# mine\ncopy = Super+Alt+C\n"
    );
    assert!(
        !dir.join("config/keycap/keymap.part").exists(),
        "no temporary file is left"
    );
    assert_eq!(files.read().person.text, "# mine\ncopy = Super+Alt+C\n");
    files.save_person("").expect("saved over");
    assert_eq!(std::fs::read_to_string(&path).expect("the file"), "");
}

#[test]
fn a_file_that_is_not_text_or_too_large_comes_back_with_a_reason() {
    let dir = scratch("keymap-trouble");
    let files = files(&dir);
    std::fs::create_dir_all(dir.join("config/keycap")).expect("mkdir");
    std::fs::write(dir.join("config/keycap/keymap"), [0xff, 0xfe]).expect("write");
    let person = files.read().person;
    assert_eq!(person.text, "");
    assert!(
        person
            .trouble
            .is_some_and(|why| why.contains("not plain text"))
    );
    std::fs::write(dir.join("config/keycap/keymap"), "#".repeat(33 * 1024)).expect("write");
    let person = files.read().person;
    assert!(person.trouble.is_some_and(|why| why.contains("32 KiB")));
}

#[test]
fn the_stamp_changes_when_either_file_does() {
    let dir = scratch("keymap-stamp");
    let files = files(&dir);
    let none = files.stamp();
    assert_eq!(files.stamp(), none, "nothing moved");
    files.save_person("copy = none\n").expect("saved");
    let person = files.stamp();
    assert_ne!(person, none, "the person's file appeared");
    std::fs::create_dir_all(dir.join("etc")).expect("mkdir");
    std::fs::write(dir.join("etc/keymap"), "find = none\n").expect("write");
    assert_ne!(files.stamp(), person, "this computer's file appeared");
}

#[test]
fn an_edit_goes_through_the_model_to_the_file_and_back_without_losing_a_line() {
    let dir = scratch("keymap-edit");
    let files = files(&dir);
    files
        .save_person("# mine\nteleport = F5\ncopy = Super+Alt+C\n")
        .expect("seed");
    let state = shortcuts_step(
        ShortcutsState::standard(),
        ShortcutsInput::Read(files.read()),
    )
    .0;
    let (state, effects) = shortcuts_step(state, ShortcutsInput::Reset(StandardAction::Copy));
    let [ShortcutsEffect::Save(text)] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    files.save_person(text).expect("saved");
    assert_eq!(files.read().person.text, "# mine\nteleport = F5\n");
    let reread = shortcuts_step(state, ShortcutsInput::Read(files.read())).0;
    let copy = reread
        .sections()
        .into_iter()
        .flat_map(|section| section.rows)
        .find(|row| row.name == "Copy")
        .expect("the Copy row");
    assert_eq!(copy.origin, Origin::Standard);
}
