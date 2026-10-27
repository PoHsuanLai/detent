//! The programs and their powers, read from the rows the compositor lists. A row's path is
//! `privacy.<power>.<id>`; beside a person's answer sits `privacy.<power>.<id>.forget`, the action
//! that puts the question back. A power nobody has answered for a program the compositor knows
//! stands at `ask` in a switch of the same shape, without that action: switching it on or off is
//! the person's first answer. An administrator's answer is a read-out that nobody can change.

use super::power::PrivacyPower;
use crate::picker::Described;
use ds_settings::schema::{KeyKind, KeyPath, KeySpec};
use std::collections::BTreeMap;

/// Before the power in a row's path.
const PREFIX: &str = "privacy.";

/// After a row's id, in the path of its "ask again" action.
const FORGET: &str = ".forget";

/// The word of a row's value that says the power is allowed.
pub(super) const ALLOWED_WORD: &str = "on";

/// The word of a row's value that says the power is denied.
pub(super) const DENIED_WORD: &str = "off";

/// The word of a row's value that says nobody has answered: the program asks first.
pub(super) const ASK_WORD: &str = "ask";

/// What the compositor calls a program: its path on disk, or the id of its sandboxed app.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramSubject(pub String);

/// What kind of thing a subject names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    /// A program, named by the path of its image.
    PrivacyProgram,
    /// A sandboxed app, named by its id.
    Sandboxed,
}

impl ProgramSubject {
    /// A path names a program; anything else is a sandboxed app's id.
    pub fn kind(&self) -> SubjectKind {
        if self.0.starts_with('/') {
            SubjectKind::PrivacyProgram
        } else {
            SubjectKind::Sandboxed
        }
    }
}

/// What a power stands at for a program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerStanding {
    /// The program may use it.
    Allowed,
    /// The program may not.
    Denied,
    /// Nobody has answered: the program asks first.
    Ask,
}

impl PowerStanding {
    /// Every standing a person can pick, in the order of the menu.
    pub const CHOICES: [PowerStanding; 3] = [
        PowerStanding::Allowed,
        PowerStanding::Denied,
        PowerStanding::Ask,
    ];
}

/// Who an answer is from, and what changes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellKind {
    /// Nobody has answered and the compositor lists no row to change: it does not know the
    /// program yet, so there is nothing to switch until the program asks.
    Asking,
    /// The person's answer: the row that allows or denies, and the action that asks again. A
    /// power nobody has answered is this too, standing at `Ask` with no action to ask again;
    /// the first switch is the person's first answer.
    Remembered {
        /// The row that is on or off.
        switch: KeyPath,
        /// The action that forgets the answer; none when the module lists none.
        forget: Option<KeyPath>,
    },
    /// An administrator's answer, which nobody can change here.
    Fixed,
}

/// One power of one program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerCell {
    /// Which power.
    pub power: PrivacyPower,
    /// Where it stands.
    pub standing: PowerStanding,
    /// Whose answer it is.
    pub kind: CellKind,
}

impl PowerCell {
    /// Whether an administrator set it, so it cannot be changed here.
    pub fn locked(&self) -> bool {
        self.kind == CellKind::Fixed
    }

    /// Whether the person can pick another standing: only their own remembered answer.
    pub fn editable(&self) -> bool {
        matches!(self.kind, CellKind::Remembered { .. })
    }
}

/// A program and all seven powers, the ones nobody answered standing at `Ask`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacyProgram {
    /// Who it is.
    pub subject: ProgramSubject,
    /// Every power, in the page's order.
    pub cells: Vec<PowerCell>,
}

impl PrivacyProgram {
    /// The cell of `power`.
    pub fn cell(&self, power: PrivacyPower) -> Option<&PowerCell> {
        self.cells.iter().find(|cell| cell.power == power)
    }
}

/// The page's programs, in the order the compositor first lists them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrivacyPane {
    /// Every program that has an answer.
    pub programs: Vec<PrivacyProgram>,
}

/// A row as the path and the schema give it.
struct Answer {
    power: PrivacyPower,
    subject: ProgramSubject,
    standing: PowerStanding,
    kind: CellKind,
}

/// What a path under `privacy.` names.
enum RowPath<'a> {
    /// An answer: the power and the row's id.
    Answer(PrivacyPower, &'a str),
    /// The action that forgets the answer with this id.
    Forget(PrivacyPower, &'a str),
}

fn parse(path: &str) -> Option<RowPath<'_>> {
    let (word, rest) = path.strip_prefix(PREFIX)?.split_once('.')?;
    let power = PrivacyPower::of_word(word)?;
    if rest.contains('.') {
        let id = rest.strip_suffix(FORGET)?;
        (!id.contains('.')).then_some(RowPath::Forget(power, id))
    } else {
        Some(RowPath::Answer(power, rest))
    }
}

fn standing_of(word: &str) -> Option<PowerStanding> {
    match word {
        ALLOWED_WORD => Some(PowerStanding::Allowed),
        DENIED_WORD => Some(PowerStanding::Denied),
        ASK_WORD => Some(PowerStanding::Ask),
        _ => None,
    }
}

impl PrivacyPane {
    /// The programs the rows of `described` list.
    pub fn of(described: &Described) -> PrivacyPane {
        let forgets: BTreeMap<(PrivacyPower, &str), &KeyPath> = described
            .keys
            .iter()
            .filter_map(|spec| match parse(&spec.path.0) {
                Some(RowPath::Forget(power, id)) => Some(((power, id), &spec.path)),
                _ => None,
            })
            .collect();
        let answers: Vec<Answer> = described
            .keys
            .iter()
            .filter_map(|spec| answer_of(spec, described, &forgets))
            .collect();
        let mut subjects: Vec<&ProgramSubject> = Vec::new();
        for answer in &answers {
            if !subjects.contains(&&answer.subject) {
                subjects.push(&answer.subject);
            }
        }
        PrivacyPane {
            programs: subjects
                .into_iter()
                .map(|subject| program_of(subject, &answers))
                .collect(),
        }
    }
}

fn answer_of(
    spec: &KeySpec,
    described: &Described,
    forgets: &BTreeMap<(PrivacyPower, &str), &KeyPath>,
) -> Option<Answer> {
    let Some(RowPath::Answer(power, id)) = parse(&spec.path.0) else {
        return None;
    };
    let standing = standing_of(described.values.get(&spec.path)?)?;
    let kind = match spec.kind {
        KeyKind::Toggle { .. } => CellKind::Remembered {
            switch: spec.path.clone(),
            forget: forgets.get(&(power, id)).map(|path| (*path).clone()),
        },
        // An administrator never leaves a power open: a read-out that says `ask` is not theirs.
        KeyKind::Fixed { .. } if standing != PowerStanding::Ask => CellKind::Fixed,
        _ => return None,
    };
    Some(Answer {
        power,
        subject: ProgramSubject(spec.label.0.clone()),
        standing,
        kind,
    })
}

/// `subject` with all seven powers: an administrator's answer over the person's, else the
/// person's, else `Ask`.
fn program_of(subject: &ProgramSubject, answers: &[Answer]) -> PrivacyProgram {
    let cells = PrivacyPower::ALL
        .into_iter()
        .map(|power| {
            let theirs: Vec<&Answer> = answers
                .iter()
                .filter(|answer| answer.power == power && answer.subject == *subject)
                .collect();
            let chosen = theirs
                .iter()
                .find(|answer| answer.kind == CellKind::Fixed)
                .or_else(|| theirs.first())
                .copied();
            match chosen {
                Some(answer) => PowerCell {
                    power,
                    standing: answer.standing,
                    kind: answer.kind.clone(),
                },
                None => PowerCell {
                    power,
                    standing: PowerStanding::Ask,
                    kind: CellKind::Asking,
                },
            }
        })
        .collect();
    PrivacyProgram {
        subject: subject.clone(),
        cells,
    }
}
