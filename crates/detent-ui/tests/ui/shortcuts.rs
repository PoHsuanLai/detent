//! The Keyboard page's shortcuts over a scratch home: the groups and who set each chord, Clear,
//! Reset and Reset All writing only the person's own keymap file, and the list following that
//! file when it changes elsewhere.

use crate::support::Sandbox;
use crate::support::drive::{click, scroll_into_view};
use chordkit::{Chord, Desktop, Platform};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::{Driver, Harness, Query};
use std::time::Duration;

const KEYMAP: &str = "keycap/keymap";
const COPY_CELL: &str = ".shortcut-cell[data-shortcut=\"Copy\"]";
const RESET_COPY: &str = "[aria-label=\"Reset Copy\"]";
const RESET_PASTE: &str = "[aria-label=\"Reset Paste\"]";
const CLEAR_COPY: &str = "[aria-label=\"Clear Copy\"]";
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
    scroll_into_view(&mut harness, COPY_CELL);
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
    let cell = harness.text_of(COPY_CELL).unwrap_or_default();
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
    let cell = harness.text_of(COPY_CELL).unwrap_or_default();
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
fn clear_writes_none_for_the_action() {
    let (sandbox, mut harness) = open("shortcuts-clear", None);
    assert_eq!(sandbox.read(KEYMAP), None, "nothing is written by looking");
    scroll_into_view(&mut harness, CLEAR_COPY);
    click(&mut harness, CLEAR_COPY);
    assert_eq!(sandbox.read(KEYMAP).as_deref(), Some("copy = none\n"));
    let cell = harness.text_of(COPY_CELL).unwrap_or_default();
    assert!(cell.contains("None"), "{cell}");
    assert_eq!(
        harness.count(RESET_COPY),
        1,
        "a cleared shortcut can be reset"
    );
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
    harness.advance(Duration::from_secs(3));
    assert_eq!(harness.count(RESET_COPY), 1, "{}", harness.html());
}

#[test]
fn a_line_that_cannot_be_used_is_said_and_kept() {
    let (sandbox, harness) = open("shortcuts-problem", Some("teleport = F5\n"));
    assert!(harness.html().contains("Your shortcuts: line 1: "));
    assert_eq!(sandbox.read(KEYMAP).as_deref(), Some("teleport = F5\n"));
}
