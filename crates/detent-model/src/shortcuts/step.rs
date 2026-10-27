//! The list's one transition: `(state, input) -> (state, effects)`. A chord another action has is
//! asked about first; the person's keymap is written only for what the person chose, and only
//! to the person's own file.

use super::model::{
    KeymapLayer, KeymapTexts, OURS, ShortcutProblem, ShortcutReplacing, ShortcutsState,
};
use super::person_text::render_person;
use chordkit::{
    Action, Chord, Context, DefaultChord, OverrideEntry, OverrideProblem, Overrides,
    StandardAction, reserved_reason,
};

/// What happened to the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutsInput {
    /// The keymap files were read, the first time or after they changed.
    Read(KeymapTexts),
    /// The person recorded a chord for an action.
    Set(StandardAction, Chord),
    /// The person cleared an action's shortcut.
    Clear(StandardAction),
    /// The person put an action back to the standard.
    Reset(StandardAction),
    /// The person put every shortcut back to the standard.
    ResetAll,
    /// The person answered Replace.
    Replace,
    /// The person answered Cancel.
    Cancel,
    /// The write ended.
    Saved(Result<(), String>),
}

/// What the caller must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutsEffect {
    /// Write `text` as the person's keymap file, if the file still holds `base`, the text it was
    /// read as: a file another program changed since is left as it is.
    Save {
        /// The file's text when the edit began.
        base: String,
        /// The whole new text.
        text: String,
    },
    /// Read the files again.
    Reread,
    /// Tell the person something, in the sentence a service gave.
    Tell(String),
    /// Tell the person the file was left alone, for this `trouble`.
    FileLeftAlone {
        /// Why the file could not be read.
        trouble: String,
    },
    /// Tell the person the desktop keeps `chord` for itself, for this `reason`.
    Reserved {
        /// The chord, as this desktop shows it.
        chord: String,
        /// Why it cannot be used.
        reason: String,
    },
}

type Step = (ShortcutsState, Vec<ShortcutsEffect>);

fn problems_in(
    layer: KeymapLayer,
    trouble: Option<&str>,
    problems: Vec<String>,
) -> Vec<ShortcutProblem> {
    trouble
        .map(str::to_owned)
        .into_iter()
        .chain(problems)
        .map(|what| ShortcutProblem { layer, what })
        .collect()
}

/// The skipped lines and the refused chords, in the words of whatever read them.
fn words_of(bad: &[OverrideProblem], refused: &[OverrideProblem]) -> Vec<String> {
    bad.iter().chain(refused).map(ToString::to_string).collect()
}

fn read(state: ShortcutsState, texts: KeymapTexts) -> ShortcutsState {
    let machine = texts.machine.overrides;
    let (person, person_bad) = Overrides::parse(&texts.person.text);
    let (under, machine_refused) = state.base.clone().with_overrides(&machine);
    let (_, person_refused) = under.with_overrides(&person);
    let mut problems = problems_in(
        KeymapLayer::Machine,
        None,
        texts
            .machine
            .problems
            .into_iter()
            .chain(words_of(&[], &machine_refused))
            .collect(),
    );
    problems.extend(problems_in(
        KeymapLayer::Person,
        texts.person.trouble.as_deref(),
        words_of(&person_bad, &person_refused),
    ));
    ShortcutsState {
        machine,
        person,
        person_text: texts.person.text,
        person_trouble: texts.person.trouble,
        problems,
        asking: None,
        ..state
    }
}

/// The changes without their line numbers, to tell whether two layers say the same.
fn shape(layer: &Overrides) -> Vec<(Action, Vec<DefaultChord>)> {
    layer
        .entries()
        .iter()
        .map(|entry| (entry.action.clone(), entry.chords.clone()))
        .collect()
}

/// `layer` with `action` set to `chords` (none unbinds it).
fn with_entry(layer: &Overrides, action: Action, chords: Vec<DefaultChord>) -> Overrides {
    let kept = layer.entries().iter().cloned();
    Overrides::from_entries(kept.chain([OverrideEntry {
        line: 0,
        action,
        chords,
    }]))
}

/// `layer` without the changes to standard actions.
fn without_standard(layer: &Overrides) -> Overrides {
    Overrides::from_entries(
        layer
            .entries()
            .iter()
            .filter(|entry| !matches!(entry.action, Action::Standard(_)))
            .cloned(),
    )
}

/// `layer` without the change to `action`.
fn without(layer: &Overrides, action: &Action) -> Overrides {
    Overrides::from_entries(
        layer
            .entries()
            .iter()
            .filter(|entry| entry.action != *action)
            .cloned(),
    )
}

/// The state with `person` as the person's keymap, and the write that makes it so; nothing when
/// it says what the person's keymap already says.
fn commit(state: ShortcutsState, person: Overrides) -> Step {
    if let Some(trouble) = &state.person_trouble {
        let effect = ShortcutsEffect::FileLeftAlone {
            trouble: trouble.clone(),
        };
        return (state, vec![effect]);
    }
    if shape(&person) == shape(&state.person) {
        return (state, vec![]);
    }
    let text = render_person(&state.person_text, &person);
    let base = state.person_text.clone();
    let state = ShortcutsState {
        person,
        person_text: text.clone(),
        asking: None,
        ..state
    };
    (state, vec![ShortcutsEffect::Save { base, text }])
}

fn set(state: ShortcutsState, action: StandardAction, chord: Chord) -> Step {
    if let Some(reason) = reserved_reason(OURS, chord) {
        let effect = ShortcutsEffect::Reserved {
            chord: chord.display(OURS),
            reason: reason.to_string(),
        };
        return (state, vec![effect]);
    }
    let live = state.live();
    let mine = Action::Standard(action);
    if live.chords_of(&mine) == [chord] {
        return (state, vec![]);
    }
    let holder = live
        .bindings(Context::Normal)
        .iter()
        .find(|(holder, bound)| *bound == chord && *holder != mine)
        .map(|(holder, _)| holder.clone());
    if let Some(holder) = holder {
        let asking = Some(ShortcutReplacing {
            action,
            chord,
            holder,
        });
        (ShortcutsState { asking, ..state }, vec![])
    } else {
        let person = with_entry(&state.person, mine, vec![DefaultChord::from(chord)]);
        commit(state, person)
    }
}

fn replace(state: ShortcutsState) -> Step {
    let Some(asking) = state.asking.clone() else {
        return (state, vec![]);
    };
    let state = ShortcutsState {
        asking: None,
        ..state
    };
    let left: Vec<DefaultChord> = state
        .live()
        .chords_of(&asking.holder)
        .into_iter()
        .filter(|chord| *chord != asking.chord)
        .map(DefaultChord::from)
        .collect();
    let person = with_entry(&state.person, asking.holder, left);
    let person = with_entry(
        &person,
        Action::Standard(asking.action),
        vec![DefaultChord::from(asking.chord)],
    );
    commit(state, person)
}

/// The state after `input`, and what to do about it.
pub fn shortcuts_step(state: ShortcutsState, input: ShortcutsInput) -> Step {
    match input {
        ShortcutsInput::Read(texts) => (read(state, texts), vec![]),
        ShortcutsInput::Set(action, chord) => set(state, action, chord),
        ShortcutsInput::Clear(action)
            if state.live().chords_of(&Action::Standard(action)).is_empty() =>
        {
            (state, vec![])
        }
        ShortcutsInput::Clear(action) => {
            let person = with_entry(&state.person, Action::Standard(action), vec![]);
            commit(state, person)
        }
        ShortcutsInput::Reset(action) => {
            let person = without(&state.person, &Action::Standard(action));
            commit(state, person)
        }
        ShortcutsInput::ResetAll => {
            let person = without_standard(&state.person);
            commit(state, person)
        }
        ShortcutsInput::Replace => replace(state),
        ShortcutsInput::Cancel => (
            ShortcutsState {
                asking: None,
                ..state
            },
            vec![],
        ),
        ShortcutsInput::Saved(Ok(())) => (state, vec![]),
        ShortcutsInput::Saved(Err(why)) => (
            state,
            vec![ShortcutsEffect::Tell(why), ShortcutsEffect::Reread],
        ),
    }
}
