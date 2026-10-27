//! How the person's file is written: their lines kept as they wrote them.

use super::*;
use crate::shortcuts::person_text::render_person;

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
