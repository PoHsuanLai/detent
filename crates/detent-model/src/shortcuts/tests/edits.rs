//! Every transition: what an edit writes, what it asks first and what it refuses.

use super::*;
use crate::shortcuts::replace_question;
use chordkit::Action;

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
        row(&asked, StandardAction::Copy).clash,
        Some(Action::Standard(StandardAction::Paste))
    );
    let question = replace_question(&asked).expect("a question");
    assert_eq!(question.holder, Action::Standard(StandardAction::Paste));
    assert_eq!(question.wanted, StandardAction::Copy);
    assert!(question.chord.contains('V'), "{question:?}");

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
    let [ShortcutsEffect::Reserved { chord, reason }] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    assert!(!chord.is_empty() && !reason.is_empty(), "{effects:?}");
    assert_eq!(row(&state, StandardAction::Copy).origin, Origin::Standard);
}

#[test]
fn a_file_that_could_not_be_read_is_never_written_over() {
    let mut unreadable = texts("", "");
    unreadable.person.trouble = Some("It is larger than 32 KiB.".to_owned());
    let (state, _) = shortcuts_step(ShortcutsState::standard(), ShortcutsInput::Read(unreadable));
    let (_, effects) = shortcuts_step(state, set_free());
    assert_eq!(
        effects,
        [ShortcutsEffect::FileLeftAlone {
            trouble: "It is larger than 32 KiB.".to_owned()
        }]
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
