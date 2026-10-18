//! The Keyboard page's shortcuts over a scratch home: the groups and who set each chord, Clear,
//! Reset and Reset All writing only the person's own keymap file, and the list following that
//! file when it changes elsewhere.

use crate::support::Sandbox;
use crate::support::drive::{click, scroll_into_view};
use chordkit::{Chord, Desktop, Overrides, Platform};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::ShortcutKey;
use ds_harness::{Harness, Input};
use std::time::Duration;

const KEYMAP: &str = "keycap/keymap";
const ALERT: &str = ".ds-alert";
const REPLACE: &str = ".ds-alert .ds-button[data-answers=\"return\"]";
const CANCEL: &str = ".ds-alert .ds-button[data-answers=\"escape\"]";
const CLASH: &str = ".ds-shortcut-field-clash";
const RESET_COPY: &str = "[aria-label=\"Reset Copy\"]";
const RESET_PASTE: &str = "[aria-label=\"Reset Paste\"]";
const RESET_ALL: &str = "[aria-label=\"Reset all shortcuts to standard\"]";

const OURS: Platform = Platform::Linux {
    desktop: Desktop::Ours,
};

fn keyboard() -> Opening {
    Opening {
        page: Some(PageSlug("keyboard_and_shortcuts".to_owned())),
        query: String::new(),
    }
}

/// The recorder of the row called `name`.
fn well(name: &str) -> String {
    format!(".ds-shortcut-field-well[aria-label=\"{name}\"]")
}

/// Clicks the recorder of `name` and gives it a key.
fn record(harness: &mut Harness, name: &str, mods: &[ShortcutKey], key: ShortcutKey) {
    scroll_into_view(harness, &well(name));
    click(harness, &well(name));
    harness.send(Input::chord(mods, key));
    harness.advance(Duration::from_millis(100));
}

fn shown(text: &str) -> String {
    let chord: Chord = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
    chord.display(OURS)
}

fn open(name: &str, person: Option<&str>) -> (Sandbox, Harness) {
    let sandbox = Sandbox::new(name);
    if let Some(text) = person {
        sandbox.write(KEYMAP, text);
    }
    let harness = sandbox.harness(keyboard());
    (sandbox, harness)
}

#[test]
fn the_page_lists_the_groups_and_says_who_set_each_chord() {
    let (_sandbox, mut harness) = open("shortcuts-list", Some("copy = Super+Alt+C\n"));
    scroll_into_view(&mut harness, &well("Copy"));
    let html = harness.html();
    for text in [
        "Launcher",
        "Apps and windows",
        "Spaces",
        "Screenshots",
        "Select all",
        "Switch to Space 3",
        "Reset All",
    ] {
        assert!(html.contains(text), "{text}");
    }
    let cell = harness.text_of(&well("Copy")).unwrap_or_default();
    assert!(cell.contains(&shown("Super+Alt+C")), "{cell}");
    assert_eq!(
        harness.count(RESET_COPY),
        1,
        "the person's own row can be reset"
    );
    assert_eq!(
        harness.count(RESET_PASTE),
        0,
        "a standard row has nothing to reset"
    );
}

#[test]
fn reset_puts_one_shortcut_back_and_touches_only_the_persons_file() {
    let (sandbox, mut harness) = open(
        "shortcuts-reset",
        Some("# mine\ncopy = Super+Alt+C\nfind = Super+Alt+F\n"),
    );
    scroll_into_view(&mut harness, RESET_COPY);
    click(&mut harness, RESET_COPY);
    let written = sandbox.read(KEYMAP).expect("the person's file");
    assert!(written.starts_with("# mine\n"), "{written}");
    assert!(written.contains("find = "), "{written}");
    assert!(!written.contains("copy"), "{written}");
    assert_eq!(harness.count(RESET_COPY), 0);
    let cell = harness.text_of(&well("Copy")).unwrap_or_default();
    assert!(
        cell.contains(&shown("Super+C")),
        "the standard chord is back: {cell}"
    );
    assert_eq!(
        sandbox.read("keycap/keymap.part"),
        None,
        "no temporary file is left"
    );
}

#[test]
fn recording_a_chord_writes_it_for_the_action() {
    let (sandbox, mut harness) = open("shortcuts-record", None);
    assert_eq!(sandbox.read(KEYMAP), None, "nothing is written by looking");
    let mods = [ShortcutKey::Super, ShortcutKey::Shift];
    record(&mut harness, "Copy", &mods, ShortcutKey::Char('k'));
    let written = sandbox.read(KEYMAP).expect("the person's file");
    let (saved, problems) = Overrides::parse(&written);
    assert!(problems.is_empty(), "{problems:?}");
    let chords: Vec<Chord> = saved.entries()[0].chords.iter().map(|c| c.base()).collect();
    assert_eq!(saved.entries()[0].action.id(), "copy");
    assert_eq!(chords, ["Shift+Super+K".parse::<Chord>().expect("a chord")]);
    assert_eq!(harness.count(RESET_COPY), 1, "a changed row can be reset");
}

#[test]
fn clearing_writes_none_for_the_action() {
    let (sandbox, mut harness) = open("shortcuts-clear", None);
    record(&mut harness, "Copy", &[], ShortcutKey::Backspace);
    assert_eq!(sandbox.read(KEYMAP).as_deref(), Some("copy = none\n"));
    let well = harness.text_of(&well("Copy")).unwrap_or_default();
    assert_eq!(well, "Add shortcut");
}

#[test]
fn a_chord_another_action_has_is_asked_about_and_cancel_changes_nothing() {
    let (sandbox, mut harness) = open("shortcuts-cancel", None);
    record(
        &mut harness,
        "Copy",
        &[ShortcutKey::Super],
        ShortcutKey::Char('v'),
    );
    assert_eq!(harness.count(ALERT), 1, "{}", harness.html());
    assert_eq!(
        sandbox.read(KEYMAP),
        None,
        "nothing is written before the answer"
    );
    let used = harness.text_of(CLASH).unwrap_or_default();
    assert!(used.contains("Used by Paste"), "{used}");
    click(&mut harness, CANCEL);
    assert_eq!(harness.count(ALERT), 0);
    assert_eq!(sandbox.read(KEYMAP), None);
    let copy = harness.text_of(&well("Copy")).unwrap_or_default();
    assert!(copy.contains(&shown("Super+C")), "{copy}");
}

#[test]
fn replace_gives_the_chord_to_the_new_action_and_takes_it_from_the_old() {
    let (sandbox, mut harness) = open("shortcuts-replace", None);
    record(
        &mut harness,
        "Copy",
        &[ShortcutKey::Super],
        ShortcutKey::Char('v'),
    );
    click(&mut harness, REPLACE);
    assert_eq!(harness.count(ALERT), 0);
    let written = sandbox.read(KEYMAP).expect("the person's file");
    assert!(written.contains("paste = none"), "{written}");
    assert!(written.contains("copy = "), "{written}");
    let copy = harness.text_of(&well("Copy")).unwrap_or_default();
    assert!(copy.contains(&shown("Super+V")), "{copy}");
    let paste = harness.text_of(&well("Paste")).unwrap_or_default();
    assert_eq!(paste, "Add shortcut");
}

#[test]
fn reset_all_empties_the_persons_changes_and_keeps_their_comments() {
    let (sandbox, mut harness) = open(
        "shortcuts-reset-all",
        Some("# mine\ncopy = Super+Alt+C\nfind = Super+Alt+F\n"),
    );
    scroll_into_view(&mut harness, RESET_ALL);
    click(&mut harness, RESET_ALL);
    assert_eq!(sandbox.read(KEYMAP).as_deref(), Some("# mine\n"));
    assert_eq!(harness.count(RESET_COPY), 0);
}

#[test]
fn the_list_follows_the_file_when_it_changes_elsewhere() {
    let (sandbox, mut harness) = open("shortcuts-follow", None);
    assert_eq!(harness.count(RESET_COPY), 0);
    sandbox.write(KEYMAP, "copy = Super+Alt+C\n");
    // The watch looks on its own thread, on real time: give it a generous while.
    for _ in 0..200 {
        if harness.count(RESET_COPY) == 1 {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("the list never followed the file:\n{}", harness.html());
}

#[test]
fn a_line_that_cannot_be_used_is_said_and_kept() {
    let (sandbox, harness) = open("shortcuts-problem", Some("teleport = F5\n"));
    assert!(harness.html().contains("Your shortcuts: line 1: "));
    assert_eq!(sandbox.read(KEYMAP).as_deref(), Some("teleport = F5\n"));
}
