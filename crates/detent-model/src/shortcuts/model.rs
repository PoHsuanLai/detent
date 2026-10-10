//! What the Shortcuts list holds: the layers of the keymap and the rows drawn from them.

use super::groups::ShortcutGroup;
use super::names::name_of;
use chordkit::{Action, Chord, Desktop, Keymap, Overrides, Platform, StandardAction};

/// The platform whose keymap the list shows: this desktop's.
pub(super) const OURS: Platform = Platform::Linux {
    desktop: Desktop::Ours,
};

/// One keymap file as read: its text, and why it could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeymapText {
    /// The file's text; empty when the file is not there or could not be read.
    pub text: String,
    /// Why the file could not be read, in plain words.
    pub trouble: Option<String>,
}

/// This computer's keymap, as keycap reads it: already parsed, and what was wrong with it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MachineLayer {
    /// The changes.
    pub overrides: Overrides,
    /// Skipped or refused lines, in plain words.
    pub problems: Vec<String>,
}

/// The two keymap layers that sit over the standard shortcuts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeymapTexts {
    /// This computer's, for everyone who uses it.
    pub machine: MachineLayer,
    /// The person's own.
    pub person: KeymapText,
}

/// Where a row's chord comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// The desktop's own.
    Standard,
    /// This computer's keymap.
    ThisComputer,
    /// The person's own keymap.
    You,
}

impl Origin {
    /// How the row says it.
    pub fn label(self) -> &'static str {
        match self {
            Origin::Standard => "Standard",
            Origin::ThisComputer => "This computer",
            Origin::You => "You",
        }
    }
}

/// One shortcut of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutRow {
    /// The action.
    pub action: StandardAction,
    /// Its name.
    pub name: String,
    /// Its first chord, `None` when nothing triggers it.
    pub chord: Option<Chord>,
    /// The chord as this desktop shows it; empty when there is none.
    pub shown: String,
    /// The chords after the first, shown.
    pub also: Vec<String>,
    /// Who set it.
    pub origin: Origin,
    /// The name of the action that has the chord this row was just given, while the person is
    /// asked whether to take it over.
    pub clash: Option<String>,
}

impl ShortcutRow {
    /// The line under the name: who set the chord, and the other chords the action has.
    pub fn note(&self) -> String {
        match self.also.as_slice() {
            [] => self.origin.label().to_owned(),
            also => format!("{}. Also {}.", self.origin.label(), also.join(", ")),
        }
    }
}

/// One titled group of rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutSection {
    /// The group.
    pub group: ShortcutGroup,
    /// Its rows.
    pub rows: Vec<ShortcutRow>,
}

/// A chord the person gave to an action that another action has: nothing is written until the
/// question is answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Replacing {
    pub(super) action: StandardAction,
    pub(super) chord: Chord,
    pub(super) holder: Action,
}

/// The words of the question asked before a chord is taken over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceQuestion {
    /// The question.
    pub title: String,
    /// What follows from answering Replace.
    pub message: String,
}

/// The desktop's standard keymap: chordkit's conventions for our desktop with `system` (the
/// desktop's own bindings) laid over them.
pub fn desktop_keymap(system: Vec<SystemBinding>) -> Keymap {
    Keymap::conventional(OURS).with_system(system)
}

/// What the list stands on.
#[derive(Debug, Clone)]
pub struct ShortcutsState {
    /// The conventions and the desktop's own bindings, under both files.
    pub(super) base: Keymap,
    pub(super) machine: Overrides,
    pub(super) person: Overrides,
    /// The person's file as it was read, so a write keeps its comments and the lines it could
    /// not use.
    pub(super) person_text: String,
    /// Why the person's file cannot be written over: it could not be read.
    pub(super) person_trouble: Option<String>,
    /// Lines and chords that were skipped, in plain words.
    pub(super) problems: Vec<String>,
    pub(super) asking: Option<Replacing>,
}

impl ShortcutsState {
    /// The list over `base` with no file read yet.
    pub fn over(base: Keymap) -> ShortcutsState {
        ShortcutsState {
            base,
            machine: Overrides::default(),
            person: Overrides::default(),
            person_text: String::new(),
            person_trouble: None,
            problems: Vec::new(),
            asking: None,
        }
    }

    /// The desktop's standard keymap with nothing laid over it.
    pub fn standard() -> ShortcutsState {
        ShortcutsState::over(desktop_keymap(Vec::new()))
    }

    /// The keymap in force: the standard, this computer's, then the person's.
    pub(super) fn live(&self) -> Keymap {
        let (machine, _) = self.base.clone().with_overrides(&self.machine);
        machine.with_overrides(&self.person).0
    }

    /// What was wrong in the files, in plain words: skipped lines, chords the desktop keeps for
    /// itself, a file that could not be read.
    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    /// The groups and their rows, as the keymap in force has them.
    pub fn sections(&self) -> Vec<ShortcutSection> {
        let live = self.live();
        ShortcutGroup::ALL
            .iter()
            .map(|group| ShortcutSection {
                group: *group,
                rows: group
                    .actions()
                    .into_iter()
                    .map(|action| self.row(&live, action))
                    .collect(),
            })
            .collect()
    }

    fn row(&self, live: &Keymap, action: StandardAction) -> ShortcutRow {
        let chords = live.chords_of(&Action::Standard(action));
        let shown: Vec<String> = chords.iter().map(|chord| chord.display(OURS)).collect();
        ShortcutRow {
            action,
            name: name_of(&Action::Standard(action)),
            chord: chords.first().copied(),
            shown: shown.first().cloned().unwrap_or_default(),
            also: shown.into_iter().skip(1).collect(),
            origin: self.origin_of(&Action::Standard(action)),
            clash: self
                .asking
                .as_ref()
                .filter(|asking| asking.action == action)
                .map(|asking| name_of(&asking.holder)),
        }
    }

    fn origin_of(&self, action: &Action) -> Origin {
        let has = |layer: &Overrides| layer.entries().iter().any(|e| e.action == *action);
        if has(&self.person) {
            Origin::You
        } else if has(&self.machine) {
            Origin::ThisComputer
        } else {
            Origin::Standard
        }
    }

    /// Whether the person's own keymap holds a change to a standard action.
    pub fn has_own(&self) -> bool {
        self.person
            .entries()
            .iter()
            .any(|e| matches!(e.action, Action::Standard(_)))
    }
}

/// The question to ask while a chord waits to be taken over.
pub fn replace_question(state: &ShortcutsState) -> Option<ReplaceQuestion> {
    let asking = state.asking.as_ref()?;
    let (chord, holder) = (asking.chord.display(OURS), name_of(&asking.holder));
    let wanted = name_of(&Action::Standard(asking.action));
    Some(ReplaceQuestion {
        title: format!("{chord} is already used by \u{201c}{holder}\u{201d}."),
        message: format!(
            "Use it for \u{201c}{wanted}\u{201d} instead? \u{201c}{holder}\u{201d} will lose it."
        ),
    })
}
