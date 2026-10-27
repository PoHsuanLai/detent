//! What the picker shows: one group per kind, one row per tier.

use super::choice::NONE_LABEL;
use super::local::LocalAsk;
use super::pool::{AutoPool, PoolWrite, is_pool};
use crate::Live;
use crate::accounts::Adding;
use crate::coding_agents::TURNED_OFF;
use crate::words::word_label;
use ds_settings::schema::{
    ChoicePlace, ChoiceWord, KeyKind, KeyPath, KeySpec, UnavailableReason, WordLabels,
};
use std::collections::BTreeMap;

/// The word that is Automatic, in the module's rows.
pub(crate) const AUTO: &str = "auto";

/// The prefix of every picker row's path: `ai.model.<slot>.<tier>`.
const PREFIX: &str = "ai.model.";

/// What separates the area from the slot in a row's section: `Models / Text`.
const SECTION_SPLIT: &str = " / ";

/// One run of a row's choices: the ungrouped ones, or one group's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceSection {
    /// The group's display name; `None` for the ungrouped choices at the top.
    pub group: Option<String>,
    /// The stored words, in the module's order.
    pub words: Vec<String>,
}

/// One tier of one slot: a menu of Off (the empty word), Automatic and each model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelRow {
    /// The module's key, `ai.model.<slot>.<tier>`.
    pub path: KeyPath,
    /// What the module calls the row ("Fast", "Demanding").
    pub label: String,
    /// The module's help text for the row; empty when it gave none.
    pub help: String,
    /// The choices, ungrouped first, then one section per group in first-appearance order.
    pub sections: Vec<ChoiceSection>,
    /// What the module calls each word.
    pub labels: WordLabels,
    /// The words that cannot be picked now, each with the reason shown under it.
    pub unavailable: BTreeMap<String, String>,
    /// Where the model behind each word runs, for the words the module says.
    pub places: BTreeMap<String, ChoicePlace>,
    /// The module's short line shown under the row; `None` when it gave none.
    pub note: Option<String>,
    /// The word chosen now: "", `auto` or `account/model`.
    pub value: String,
    /// The models Automatic chooses among for this row's slot; empty when the module lists none.
    pub pool: AutoPool,
    /// Whether the person is answering what Local asks, before a model of it is chosen.
    pub local: LocalAsk,
}

/// The rows of one slot, as the schema's section and labels name them.
#[derive(Debug, Clone, PartialEq)]
pub struct KindRows {
    /// The area the slot sits in, "Models": the section's part before ` / `. `None` when the
    /// section names no area.
    pub area: Option<String>,
    /// What a person reads for the slot, "Text": the section's part after ` / `.
    pub label: String,
    /// One row per tier the module lists, in its order.
    pub rows: Vec<ModelRow>,
}

/// What `Describe` and `Get` returned: the keys and each one's value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Described {
    /// The module's keys.
    pub keys: Vec<KeySpec>,
    /// The value of each key the module answered for with a word.
    pub values: BTreeMap<KeyPath, String>,
    /// The value of each key the module answered for with a list of words.
    pub lists: BTreeMap<KeyPath, Vec<String>>,
}

/// A set that is in flight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applying {
    /// The row being set.
    pub path: KeyPath,
    /// The word it is being set to.
    pub word: String,
}

/// A pick that waits for the person to add an account: of a hosted model that no account
/// reaches, or of an engine to run models on this computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwaitingAccount {
    /// The row picked in.
    pub path: KeyPath,
    /// The model picked; none when the account asked for is an engine for Local.
    pub word: Option<String>,
}

/// A pick of a coding agent that waits for the person to say yes to turning coding agents on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnOn {
    /// The row picked in.
    pub path: KeyPath,
    /// The agent's word.
    pub word: String,
}

/// What a picker that is there waits on: nothing, or exactly one of these.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Busy {
    /// Nothing: a pick may be taken.
    #[default]
    Idle,
    /// The set in flight.
    Applying(Applying),
    /// The pick that waits on an account being added.
    Awaiting(AwaitingAccount),
    /// The pool write in flight.
    Pooling(PoolWrite),
    /// The coding agent picked while coding agents are off, waiting on the person's yes to turn
    /// them on.
    TurningOn(TurnOn),
}

/// A picker that is there.
#[derive(Debug, Clone, PartialEq)]
pub struct Ready {
    /// The slots, in the order their rows first appear in `Describe`.
    pub kinds: Vec<KindRows>,
    /// What the picker waits on, if anything: one at a time, and no other pick is taken meanwhile.
    pub busy: Busy,
}

/// Where the picker stands.
pub type PickerState = Live<Ready>;

/// The slot segment of an `ai.model.<slot>.<tier>` path.
pub(super) fn slot_of(path: &KeyPath) -> Option<&str> {
    path.0
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.split_once('.'))
        .map(|(slot, _)| slot)
}

/// Whether `path` is `ai.model.<slot>.<tier>`.
fn is_model_row(path: &KeyPath) -> bool {
    path.0
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.split_once('.'))
        .is_some_and(|(slot, tier)| !slot.is_empty() && !tier.is_empty())
}

/// Where a row sits and what it is called, from the schema alone: section `Models / Text` with
/// label `Fast` is area `Models`, slot `Text`, row `Fast`. An older module sends section `Models`
/// and label `Text: Fast`: the area is the section, the slot the label's part before `: `.
fn place(spec: &KeySpec) -> (Option<String>, String, String) {
    let label = spec.label.0.as_str();
    match spec.section.0.split_once(SECTION_SPLIT) {
        Some((area, slot)) => (Some(area.to_owned()), slot.to_owned(), label.to_owned()),
        None => match label.split_once(": ") {
            Some((slot, tier)) => (
                Some(spec.section.0.clone()),
                slot.to_owned(),
                tier.to_owned(),
            ),
            None => (None, spec.section.0.clone(), label.to_owned()),
        },
    }
}

impl ModelRow {
    /// The row `spec` describes, with `value` chosen; `None` when it is not an
    /// `ai.model.<slot>.<tier>` menu.
    fn of(spec: &KeySpec, label: String, value: String) -> Option<ModelRow> {
        if !is_model_row(&spec.path) || !matches!(spec.kind, KeyKind::Menu { .. }) {
            return None;
        }
        Some(ModelRow {
            path: spec.path.clone(),
            label,
            help: spec.help.0.clone(),
            sections: sections_of(spec),
            labels: spec.labels.clone(),
            unavailable: unavailable_of(&spec.unavailable),
            places: places_of(spec),
            note: spec.note.as_ref().map(|note| note.0.clone()),
            value,
            pool: AutoPool::default(),
            local: LocalAsk::Closed,
        })
    }

    /// The reason `word` cannot be picked, if it cannot.
    pub fn unavailable_reason(&self, word: &str) -> Option<&str> {
        self.unavailable.get(word).map(String::as_str)
    }

    /// What the menu calls `word`. The empty word is always "Off": the module's own name for it
    /// would claim a model is chosen for the person.
    pub fn label_of(&self, word: &str) -> String {
        if word.is_empty() {
            return NONE_LABEL.to_owned();
        }
        word_label(&self.labels, word)
    }
}

impl Ready {
    /// The picker `described` makes: its menu rows grouped by slot, slots and rows in the order
    /// they appear in `Describe`. Nothing here knows a slot by name.
    pub fn of(described: &Described) -> Ready {
        let mut kinds: Vec<KindRows> = Vec::new();
        for spec in &described.keys {
            let value = described
                .values
                .get(&spec.path)
                .cloned()
                .unwrap_or_default();
            let (area, slot, label) = place(spec);
            let Some(row) = ModelRow::of(spec, label, value) else {
                continue;
            };
            match kinds
                .iter_mut()
                .find(|rows| rows.area == area && rows.label == slot)
            {
                Some(rows) => rows.rows.push(row),
                None => kinds.push(KindRows {
                    area,
                    label: slot,
                    rows: vec![row],
                }),
            }
        }
        for spec in described.keys.iter().filter(|spec| is_pool(spec)) {
            let words = described.lists.get(&spec.path).cloned().unwrap_or_default();
            if let Some(pool) = AutoPool::of(spec, words) {
                give_pool(&mut kinds, pool);
            }
        }
        Ready {
            kinds,
            busy: Busy::Idle,
        }
    }

    /// Whether the add-account sheet is open on behalf of a pick.
    pub fn adding(&self) -> Adding {
        if matches!(self.busy, Busy::Awaiting(_)) {
            Adding::SheetOpen
        } else {
            Adding::Idle
        }
    }

    /// Whether a pick may be taken: no set is in flight and no account is being added.
    pub(crate) fn is_idle(&self) -> bool {
        matches!(self.busy, Busy::Idle)
    }

    /// The set in flight, if that is what the picker waits on.
    pub fn applying(&self) -> Option<&Applying> {
        match &self.busy {
            Busy::Applying(applying) => Some(applying),
            _ => None,
        }
    }

    /// The pick that waits on an account, if that is what the picker waits on.
    pub fn awaiting(&self) -> Option<&AwaitingAccount> {
        match &self.busy {
            Busy::Awaiting(pick) => Some(pick),
            _ => None,
        }
    }

    /// The pool write in flight, if that is what the picker waits on.
    pub fn pooling(&self) -> Option<&PoolWrite> {
        match &self.busy {
            Busy::Pooling(write) => Some(write),
            _ => None,
        }
    }

    /// The pick that waits on the person's yes to turning coding agents on, if that is what the
    /// picker waits on.
    pub fn turning_on(&self) -> Option<&TurnOn> {
        match &self.busy {
            Busy::TurningOn(asked) => Some(asked),
            _ => None,
        }
    }

    /// The row at `path`.
    pub fn row(&self, path: &KeyPath) -> Option<&ModelRow> {
        self.kinds
            .iter()
            .flat_map(|kind| kind.rows.iter())
            .find(|row| &row.path == path)
    }

    /// Coding agents were switched on: no word is "Turned off" any more.
    pub(crate) fn agents_are_on(&mut self) {
        for row in self.kinds.iter_mut().flat_map(|kind| kind.rows.iter_mut()) {
            row.unavailable.retain(|_, reason| reason != TURNED_OFF);
        }
    }

    /// The row at `path`, to change.
    pub(crate) fn row_mut(&mut self, path: &KeyPath) -> Option<&mut ModelRow> {
        self.kinds
            .iter_mut()
            .flat_map(|kind| kind.rows.iter_mut())
            .find(|row| &row.path == path)
    }
}

/// The choices of a menu key as sections: the ungrouped ones first, then one per group.
pub(super) fn sections_of(spec: &KeySpec) -> Vec<ChoiceSection> {
    spec.grouped_choices()
        .into_iter()
        .map(|section| ChoiceSection {
            group: section.group.map(|group| group.0.clone()),
            words: section.choices.into_iter().map(str::to_owned).collect(),
        })
        .collect()
}

/// Hands `pool` to every row of the slot it belongs to.
fn give_pool(kinds: &mut [KindRows], pool: AutoPool) {
    let rows = kinds
        .iter_mut()
        .flat_map(|kind| kind.rows.iter_mut())
        .filter(|row| pool.is_for(&row.path));
    for row in rows {
        row.pool = pool.clone();
    }
}

/// The module's unavailable choices as plain words and reasons.
pub(super) fn unavailable_of(
    unavailable: &BTreeMap<ChoiceWord, UnavailableReason>,
) -> BTreeMap<String, String> {
    unavailable
        .iter()
        .map(|(word, reason)| (word.0.clone(), reason.0.clone()))
        .collect()
}

/// Where the model behind each word of `spec` runs, as plain words.
pub(super) fn places_of(spec: &KeySpec) -> BTreeMap<String, ChoicePlace> {
    spec.places
        .iter()
        .map(|(word, place)| (word.0.clone(), place.clone()))
        .collect()
}

/// What a row shows while `word` is being set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loading {
    /// One of the two words that name no model: the empty one and Automatic.
    Applying,
    /// A model, by the name the menu gives it.
    Model(String),
}

/// What the row shows while `word` is being set.
pub fn loading_of(row: &ModelRow, word: &str) -> Loading {
    if word.is_empty() || word == AUTO {
        Loading::Applying
    } else {
        Loading::Model(row.label_of(word))
    }
}
