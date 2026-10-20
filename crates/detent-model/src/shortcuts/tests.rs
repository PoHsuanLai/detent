//! The list's groups and names, its origins, every transition, and how the person's file is
//! written.

use super::person_text::render_person;
use super::{
    KeymapText, KeymapTexts, MachineLayer, Origin, ShortcutGroup, ShortcutRow, ShortcutsEffect,
    ShortcutsInput, ShortcutsState, name_of, replace_question, shortcuts_step,
};
use chordkit::{Action, Chord, Overrides, StandardAction};

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
    for group in ShortcutGroup::ALL.iter().filter(|group| group.is_desktop()) {
        for action in group.actions() {
            assert!(keycap_client::handler(action).is_some(), "{action:?}");
        }
    }
    assert!(!shown.contains(&StandardAction::EmojiAndSymbols));
    assert!(!shown.contains(&StandardAction::NextInputSource));
    let groups: Vec<ShortcutGroup> = ShortcutsState::standard()
        .sections()
        .iter()
        .map(|section| section.group)
        .collect();
    assert!(
        !groups.contains(&ShortcutGroup::Input),
        "an empty group is not drawn"
    );
}

#[test]
fn an_app_group_keeps_every_action() {
    for group in [ShortcutGroup::Text, ShortcutGroup::Documents] {
        assert_eq!(group.actions(), group.all_actions());
    }
}

#[test]
fn names_are_plain_words() {
    let three = StandardAction::all()
        .find(|a| a.slug() == "switch-to-space-3")
        .expect("the third Space");
    let cases: &[(&str, StandardAction, &str)] = &[
        (
            "the slug capitalised",
            StandardAction::SelectAll,
            "Select all",
        ),
        (
            "a longer slug",
            StandardAction::PasteAndMatchStyle,
            "Paste and match style",
        ),
        (
            "a worded name",
            StandardAction::LockScreen,
            "Lock the screen",
        ),
        ("a numbered Space", three, "Switch to Space 3"),
    ];
    for (case, action, want) in cases {
        assert_eq!(name_of(&Action::Standard(*action)), *want, "{case}");
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
    assert_eq!(
        problems[0],
        "This computer's shortcuts: It is not plain text."
    );
    assert!(
        problems[1].starts_with("Your shortcuts: line 1: "),
        "{problems:?}"
    );
    assert!(
        problems[2].starts_with("Your shortcuts: line 2: "),
        "{problems:?}"
    );
}

/// `(case, person file before, input, entries saved or None for nothing written)`.
type Case = (
    &'static str,
    &'static str,
    fn() -> ShortcutsInput,
    Option<&'static [(&'static str, &'static str)]>,
);

fn set_free() -> ShortcutsInput {
    ShortcutsInput::Set(StandardAction::Copy, chord("Super+Alt+C"))
}
fn set_same() -> ShortcutsInput {
    ShortcutsInput::Set(StandardAction::Copy, chord("Super+C"))
}
fn clear_copy() -> ShortcutsInput {
    ShortcutsInput::Clear(StandardAction::Copy)
}
fn reset_copy() -> ShortcutsInput {
    ShortcutsInput::Reset(StandardAction::Copy)
}
fn reset_all() -> ShortcutsInput {
    ShortcutsInput::ResetAll
}

const CASES: &[Case] = &[
    (
        "a free chord is written",
        "",
        set_free,
        Some(&[("copy", "Super+Alt+C")]),
    ),
    (
        "the chord it already has writes nothing",
        "",
        set_same,
        None,
    ),
    ("clear writes none", "", clear_copy, Some(&[("copy", "")])),
    (
        "reset takes the entry out",
        "copy = Super+Alt+C\nfind = Super+Alt+F\n",
        reset_copy,
        Some(&[("find", "Super+Alt+F")]),
    ),
    (
        "reset of what is not changed writes nothing",
        "find = Super+Alt+F\n",
        reset_copy,
        None,
    ),
    (
        "reset all empties the file",
        "copy = Super+Alt+C\nfind = Super+Alt+F\n",
        reset_all,
        Some(&[]),
    ),
    (
        "reset all with nothing changed writes nothing",
        "",
        reset_all,
        None,
    ),
];

#[test]
fn an_edit_writes_what_the_person_chose_and_nothing_else() {
    for (case, before, input, want) in CASES {
        let (state, effects) = shortcuts_step(read("", before), input());
        match want {
            None => assert!(effects.is_empty(), "{case}: {effects:?}"),
            Some(entries) => {
                let want: Vec<(String, Vec<Chord>)> = entries
                    .iter()
                    .map(|(id, text)| {
                        (
                            (*id).to_owned(),
                            if text.is_empty() {
                                vec![]
                            } else {
                                vec![chord(text)]
                            },
                        )
                    })
                    .collect();
                assert_eq!(saved(&effects), want, "{case}");
            }
        }
        // The state moved with the write: reading what was written back changes nothing.
        if let [ShortcutsEffect::Save { text, .. }] = effects.as_slice() {
            let again = read("", text);
            assert_eq!(again.sections(), state.sections(), "{case}");
        }
    }
}

#[test]
fn a_chord_another_action_has_is_asked_about_not_taken() {
    let (asked, effects) = shortcuts_step(
        read("", ""),
        ShortcutsInput::Set(StandardAction::Copy, chord("Super+V")),
    );
    assert!(effects.is_empty(), "nothing is written before the answer");
    assert_eq!(
        row(&asked, StandardAction::Copy).chord,
        Some(chord("Super+C"))
    );
    assert_eq!(
        row(&asked, StandardAction::Copy).clash.as_deref(),
        Some("Paste")
    );
    let question = replace_question(&asked).expect("a question");
    assert!(question.title.contains("Paste"), "{question:?}");
    assert!(question.message.contains("Copy"), "{question:?}");

    let (replaced, effects) = shortcuts_step(asked.clone(), ShortcutsInput::Replace);
    assert_eq!(
        saved(&effects),
        [
            ("paste".to_owned(), vec![]),
            ("copy".to_owned(), vec![chord("Super+V")]),
        ]
    );
    assert_eq!(
        row(&replaced, StandardAction::Copy).chord,
        Some(chord("Super+V"))
    );
    assert_eq!(row(&replaced, StandardAction::Paste).chord, None);
    assert!(replace_question(&replaced).is_none());

    let (cancelled, effects) = shortcuts_step(asked, ShortcutsInput::Cancel);
    assert!(effects.is_empty());
    assert_eq!(
        row(&cancelled, StandardAction::Paste).chord,
        Some(chord("Super+V"))
    );
    assert!(replace_question(&cancelled).is_none());
    assert_eq!(row(&cancelled, StandardAction::Copy).clash, None);
}

#[test]
fn a_chord_the_desktop_keeps_is_refused_with_its_reason() {
    let (state, effects) = shortcuts_step(
        read("", ""),
        ShortcutsInput::Set(StandardAction::Copy, chord("Super+Tab")),
    );
    let [ShortcutsEffect::Tell(words)] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert!(words.contains("can't be used"), "{words}");
    assert_eq!(row(&state, StandardAction::Copy).origin, Origin::Standard);
}

#[test]
fn a_file_that_could_not_be_read_is_never_written_over() {
    let mut unreadable = texts("", "");
    unreadable.person.trouble = Some("It is larger than 32 KiB.".to_owned());
    let (state, _) = shortcuts_step(ShortcutsState::standard(), ShortcutsInput::Read(unreadable));
    let (_, effects) = shortcuts_step(state, set_free());
    assert!(
        matches!(effects.as_slice(), [ShortcutsEffect::Tell(_)]),
        "{effects:?}"
    );
}

#[test]
fn a_failed_write_is_told_and_the_files_are_read_again() {
    let (_, effects) = shortcuts_step(
        read("", ""),
        ShortcutsInput::Saved(Err("Disk full.".to_owned())),
    );
    assert_eq!(
        effects,
        [
            ShortcutsEffect::Tell("Disk full.".to_owned()),
            ShortcutsEffect::Reread
        ]
    );
}

/// `(case, the file before, what the person's changes are, the file after)`.
const FILES: &[(&str, &str, &str, &str)] = &[
    (
        "an edited line stays in place with the comments around it",
        "# mine\ncopy = Primary+C\n\n# more\npaste = Primary+V\n",
        "copy = Primary+Alt+C\npaste = Primary+V\n",
        "# mine\ncopy = Primary+Alt+C\n\n# more\npaste = Primary+V\n",
    ),
    (
        "a removed action loses its lines and nothing else",
        "# mine\ncopy = Primary+C\ncopy = Primary+Alt+C\nfind = Primary+F\n",
        "find = Primary+F\n",
        "# mine\nfind = Primary+F\n",
    ),
    (
        "a line that could not be read is kept",
        "teleport = F5\ncopy = Primary+C\n",
        "copy = none\n",
        "teleport = F5\ncopy = none\n",
    ),
    (
        "a new action goes at the end",
        "copy = Primary+C\n",
        "copy = Primary+C\nfind = Primary+Alt+F\n",
        "copy = Primary+C\nfind = Primary+Alt+F\n",
    ),
    (
        "an empty file takes the changes",
        "",
        "copy = Primary+Alt+C\n",
        "copy = Primary+Alt+C\n",
    ),
];

#[test]
fn the_persons_file_keeps_everything_but_the_lines_that_changed() {
    for (case, before, after, want) in FILES {
        let (after, problems) = Overrides::parse(after);
        assert!(problems.is_empty(), "{case}: {problems:?}");
        assert_eq!(render_person(before, &after), *want, "{case}");
    }
}

#[test]
fn a_save_carries_the_text_the_edit_started_from() {
    let before = "# mine\ncopy = Primary+Alt+C\n";
    let (state, effects) = shortcuts_step(read("", before), ShortcutsInput::ResetAll);
    let [ShortcutsEffect::Save { base, text }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(base, before);
    assert_eq!(text, "# mine\n");
    // The next save starts from what this one wrote.
    let (_, again) = shortcuts_step(
        state,
        ShortcutsInput::Set(StandardAction::Copy, chord("Super+Alt+C")),
    );
    assert!(
        matches!(again.as_slice(), [ShortcutsEffect::Save { base, .. }] if base == "# mine\n"),
        "{again:?}"
    );
}

#[test]
fn reset_all_keeps_what_the_list_does_not_show() {
    let before = "# mine\ncopy = Primary+Alt+C\nmail.compose = Primary+Alt+N\nbogus line\n";
    let (state, effects) = shortcuts_step(read("", before), ShortcutsInput::ResetAll);
    let [ShortcutsEffect::Save { text, .. }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert_eq!(text, "# mine\nmail.compose = Primary+Alt+N\nbogus line\n");
    assert!(!state.has_own());
}
