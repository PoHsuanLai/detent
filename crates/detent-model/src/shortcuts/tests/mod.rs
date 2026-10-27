//! The list's groups, its origins and what the files could not give; the transitions are in
//! `edits`, the person's file in `file`.

mod edits;
mod file;

use super::{
    KeymapLayer, KeymapText, KeymapTexts, MachineLayer, Origin, ShortcutGroup, ShortcutRow,
    ShortcutsEffect, ShortcutsInput, ShortcutsState, shortcuts_step,
};
use chordkit::{Chord, Overrides, StandardAction};

fn chord(text: &str) -> Chord {
    text.parse().unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn texts(machine: &str, person: &str) -> KeymapTexts {
    let (overrides, bad) = Overrides::parse(machine);
    KeymapTexts {
        machine: MachineLayer {
            overrides,
            problems: bad.iter().map(ToString::to_string).collect(),
        },
        person: KeymapText {
            text: person.to_owned(),
            trouble: None,
        },
    }
}

fn read(machine: &str, person: &str) -> ShortcutsState {
    shortcuts_step(
        ShortcutsState::standard(),
        ShortcutsInput::Read(texts(machine, person)),
    )
    .0
}

fn row(state: &ShortcutsState, action: StandardAction) -> ShortcutRow {
    state
        .sections()
        .into_iter()
        .flat_map(|section| section.rows)
        .find(|row| row.action == action)
        .unwrap_or_else(|| panic!("{action:?} has no row"))
}

/// The entries of a saved file as `(action id, chords)`.
fn saved(effects: &[ShortcutsEffect]) -> Vec<(String, Vec<Chord>)> {
    let [ShortcutsEffect::Save { text, .. }] = effects else {
        panic!("one save was expected, got {effects:?}");
    };
    let (overrides, problems) = Overrides::parse(text);
    assert!(problems.is_empty(), "{problems:?}");
    overrides
        .entries()
        .iter()
        .map(|e| (e.action.id(), e.chords.iter().map(|c| c.base()).collect()))
        .collect()
}

#[test]
fn every_standard_action_is_in_exactly_one_group() {
    let listed: Vec<StandardAction> = ShortcutGroup::ALL
        .iter()
        .flat_map(|group| group.all_actions())
        .collect();
    for action in StandardAction::all() {
        let times = listed.iter().filter(|listed| **listed == action).count();
        assert_eq!(times, 1, "{action:?} is listed {times} times");
    }
    assert_eq!(listed.len(), StandardAction::all().count());
}

#[test]
fn a_desktop_group_lists_exactly_what_the_desktop_handles() {
    let handled = |action: StandardAction| action != StandardAction::EmojiAndSymbols;
    for group in ShortcutGroup::ALL {
        let listed = group.listed(handled);
        // Every action the stub refuses is a desktop one, so the app-menu rule never shows here.
        for action in group.all_actions() {
            let expected = !group.is_desktop() || handled(action);
            assert_eq!(
                listed.contains(&action),
                expected,
                "{action:?} in {group:?}"
            );
        }
    }
}

#[test]
fn the_page_lists_every_handled_action_and_no_unhandled_desktop_one() {
    let shown: Vec<StandardAction> = ShortcutGroup::ALL
        .iter()
        .flat_map(|group| group.actions())
        .collect();
    for (action, _) in keycap_client::handled_actions() {
        assert!(
            shown.contains(&action),
            "{action:?} is handled but not listed"
        );
    }
    for group in ShortcutGroup::ALL {
        for action in group.actions() {
            if group.is_desktop_action(action) {
                assert!(keycap_client::handler(action).is_some(), "{action:?}");
            }
        }
    }
    // The shell handles both since keycap a66556b (shell-verbs), so the Input group lists them.
    assert!(shown.contains(&StandardAction::EmojiAndSymbols));
    assert!(shown.contains(&StandardAction::NextInputSource));
    let groups: Vec<ShortcutGroup> = ShortcutsState::standard()
        .sections()
        .iter()
        .map(|section| section.group)
        .collect();
    assert!(
        groups.contains(&ShortcutGroup::Input),
        "a group with handled actions is drawn"
    );
}

#[test]
fn an_apps_own_window_action_stays_though_the_desktop_does_not_handle_it() {
    let nothing = |_: StandardAction| false;
    let windows = ShortcutGroup::Windows.listed(nothing);
    for kept in [
        StandardAction::Hide,
        StandardAction::Minimize,
        StandardAction::Close,
        StandardAction::Quit,
    ] {
        assert!(windows.contains(&kept), "{kept:?}");
    }
    for gone in [StandardAction::NextApp, StandardAction::ForceQuit] {
        assert!(!windows.contains(&gone), "{gone:?}");
    }
}

#[test]
fn an_app_group_keeps_every_action() {
    for group in [ShortcutGroup::Text, ShortcutGroup::Documents] {
        assert_eq!(group.actions(), group.all_actions());
    }
}

#[test]
fn a_row_says_who_set_its_chord() {
    let state = read("find = Primary+Alt+F\n", "copy = Primary+Alt+C\n");
    let cases: &[(&str, StandardAction, Origin, Option<&str>)] = &[
        (
            "the person's",
            StandardAction::Copy,
            Origin::You,
            Some("Super+Alt+C"),
        ),
        (
            "this computer's",
            StandardAction::Find,
            Origin::ThisComputer,
            Some("Super+Alt+F"),
        ),
        (
            "the standard",
            StandardAction::Paste,
            Origin::Standard,
            Some("Super+V"),
        ),
    ];
    for (case, action, origin, chord_text) in cases {
        let found = row(&state, *action);
        assert_eq!(found.origin, *origin, "{case}");
        assert_eq!(found.chord, chord_text.map(chord), "{case}");
    }
}

#[test]
fn the_person_wins_over_this_computer() {
    let state = read("copy = Primary+Alt+F\n", "copy = Primary+Alt+C\n");
    let found = row(&state, StandardAction::Copy);
    assert_eq!(found.origin, Origin::You);
    assert_eq!(found.chord, Some(chord("Super+Alt+C")));
}

#[test]
fn the_files_say_what_could_not_be_used() {
    let mut broken = texts("", "copy = Primary+Nope\nredo = Super+Tab\n");
    broken.machine.problems = vec!["It is not plain text.".to_owned()];
    let (state, effects) = shortcuts_step(ShortcutsState::standard(), ShortcutsInput::Read(broken));
    assert!(effects.is_empty());
    let problems = state.problems();
    assert_eq!(problems.len(), 3, "{problems:?}");
    assert_eq!(problems[0].layer, KeymapLayer::Machine);
    assert_eq!(problems[0].what, "It is not plain text.");
    assert_eq!(problems[1].layer, KeymapLayer::Person);
    assert!(problems[1].what.starts_with("line 1: "), "{problems:?}");
    assert_eq!(problems[2].layer, KeymapLayer::Person);
    assert!(problems[2].what.starts_with("line 2: "), "{problems:?}");
}
