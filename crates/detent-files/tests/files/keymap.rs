//! The keymap files over a scratch home: read leniently, the person's written whole through a
//! rename, and the watch that tells a change.

use chordkit::StandardAction;
use detent_files::{ConfigHome, FilesError, KeymapFiles};
use detent_model::{Origin, ShortcutsEffect, ShortcutsInput, ShortcutsState, shortcuts_step};
use detent_testkit::Scratch;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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
    assert!(texts.machine.overrides.entries().is_empty());
    assert!(texts.machine.problems.is_empty());
}

#[test]
fn the_person_file_is_written_whole_in_keycaps_place_and_read_back() {
    let dir = scratch("keymap-save");
    let files = files(&dir);
    files
        .save_person("", "# mine\ncopy = Super+Alt+C\n")
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
    files
        .save_person("# mine\ncopy = Super+Alt+C\n", "")
        .expect("saved over");
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
fn this_computers_file_is_read_when_no_daemon_answers() {
    let dir = scratch("keymap-machine");
    std::fs::create_dir_all(dir.join("etc")).expect("mkdir");
    std::fs::write(
        dir.join("etc/keymap"),
        "find = Super+Alt+F\nteleport = F5\n",
    )
    .expect("write");
    let machine = files(&dir).read().machine;
    assert_eq!(machine.overrides.entries().len(), 1);
    assert_eq!(machine.problems.len(), 1, "{:?}", machine.problems);
}

#[test]
fn the_watch_fires_when_the_persons_file_changes() {
    let dir = scratch("keymap-watch");
    let files = files(&dir);
    let fired = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&fired);
    let _watch = files
        .watch(move || {
            seen.fetch_add(1, Ordering::SeqCst);
        })
        .expect("a watch");
    files.save_person("", "copy = none\n").expect("saved");
    // The watch looks on its own thread at its own pace.
    for _ in 0..200 {
        if fired.load(Ordering::SeqCst) > 0 {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("the watch never fired");
}

#[test]
fn an_edit_goes_through_the_model_to_the_file_and_back_without_losing_a_line() {
    let dir = scratch("keymap-edit");
    let files = files(&dir);
    files
        .save_person("", "# mine\nteleport = F5\ncopy = Super+Alt+C\n")
        .expect("seed");
    let state = shortcuts_step(
        ShortcutsState::over(files.base()),
        ShortcutsInput::Read(files.read()),
    )
    .0;
    let (state, effects) = shortcuts_step(state, ShortcutsInput::Reset(StandardAction::Copy));
    let [ShortcutsEffect::Save { base, text }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    files.save_person(base, text).expect("saved");
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

#[test]
fn a_keymap_another_program_changed_since_it_was_read_is_left_as_it_is() {
    let dir = scratch("keymap-changed");
    let files = files(&dir);
    files.save_person("", "copy = none\n").expect("seed");
    // Another program rewrites the file after the list read it.
    files
        .save_person("copy = none\n", "copy = none\nfind = Super+Alt+F\n")
        .expect("the other program's save");

    let stale = files.save_person("copy = none\n", "# mine\n");
    assert!(
        matches!(stale, Err(FilesError::Changed { .. })),
        "{stale:?}"
    );
    assert_eq!(
        files.read().person.text,
        "copy = none\nfind = Super+Alt+F\n",
        "the other program's change is still there"
    );
}
