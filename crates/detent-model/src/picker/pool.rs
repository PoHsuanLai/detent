//! The models Automatic chooses among for a slot: the module's `ai.auto.pool.<slot>` list, the
//! slot picker's own choices without the empty word and Automatic.

use super::choice::{ChoiceGap, ChoiceReadiness, Gate, gate_of};
use super::model::{ChoiceSection, Ready, sections_of, slot_of, unavailable_of};
use crate::words::word_label;
use ds_settings::schema::{KeyKind, KeyPath, KeySpec, WordLabels};
use std::collections::BTreeMap;

/// The prefix of a pool's path: `ai.auto.pool.<slot>`.
const POOL_PREFIX: &str = "ai.auto.pool.";

/// What the models a pool holds but the module no longer offers are listed under.
const GONE_GROUP: &str = "No longer available";

/// Whether `spec` is a slot's pool: a list of choices from a menu.
pub(super) fn is_pool(spec: &KeySpec) -> bool {
    spec.path.0.strip_prefix(POOL_PREFIX).is_some()
        && matches!(&spec.kind, KeyKind::List(inner) if matches!(**inner, KeyKind::Menu { .. }))
}

/// The pool of one slot, as every tier's row of the slot holds it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AutoPool {
    /// The module's key; `None` when the module lists no pool for the slot.
    pub path: Option<KeyPath>,
    /// The models that can be put in the pool, ungrouped first, then one section per group.
    pub sections: Vec<ChoiceSection>,
    /// What the module calls each model.
    pub labels: WordLabels,
    /// The models that cannot be put in the pool as they are, each with the reason.
    pub unavailable: BTreeMap<String, String>,
    /// The module's short line shown under the list; `None` when it gave none.
    pub note: Option<String>,
    /// The models in the pool now, in the order they were added.
    pub chosen: Vec<String>,
}

/// Whether a model is in the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolMembership {
    /// It is in the pool.
    In,
    /// It is not.
    Out,
}

/// How a model stands in the pool editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolStanding {
    /// The module offers it, and the gate says whether it can be added.
    Offered(Gate),
    /// The pool holds it and the module no longer offers it.
    Gone,
}

/// One line of the pool editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolEntry {
    /// The stored word, `account/model`.
    pub word: String,
    /// What a person reads for it.
    pub label: String,
    /// Whether it is in the pool.
    pub membership: PoolMembership,
    /// How it stands.
    pub standing: PoolStanding,
}

impl PoolEntry {
    /// Whether the line can be turned: a model that is blocked cannot be added, and one that is
    /// gone can only be removed.
    pub fn can_toggle(&self) -> bool {
        let addable = matches!(
            self.standing,
            PoolStanding::Offered(Gate::Open | Gate::NeedsAccount)
        );
        addable || self.membership == PoolMembership::In
    }
}

/// A run of lines under one header: the ungrouped ones, or one group's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolGroup {
    /// The group's name; `None` for the ungrouped lines.
    pub group: Option<String>,
    /// The lines, in the module's order.
    pub entries: Vec<PoolEntry>,
}

/// A pool write that is in flight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolWrite {
    /// The pool's key.
    pub path: KeyPath,
    /// The models it is being set to.
    pub words: Vec<String>,
}

impl AutoPool {
    /// The pool `spec` describes, holding `chosen`; `None` when `spec` is not a list of choices.
    pub(super) fn of(spec: &KeySpec, chosen: Vec<String>) -> Option<AutoPool> {
        let KeyKind::List(inner) = &spec.kind else {
            return None;
        };
        let menu = KeySpec {
            kind: (**inner).clone(),
            ..spec.clone()
        };
        Some(AutoPool {
            path: Some(spec.path.clone()),
            sections: sections_of(&menu),
            labels: spec.labels.clone(),
            unavailable: unavailable_of(&spec.unavailable),
            note: spec.note.as_ref().map(|note| note.0.clone()),
            chosen,
        })
    }

    /// Whether this is the pool of the slot the row at `row` belongs to.
    pub(super) fn is_for(&self, row: &KeyPath) -> bool {
        let slot = self
            .path
            .as_ref()
            .and_then(|path| path.0.strip_prefix(POOL_PREFIX));
        matches!((slot, slot_of(row)), (Some(own), Some(theirs)) if own == theirs)
    }

    /// Whether the module offers `word`.
    fn offers(&self, word: &str) -> bool {
        self.sections
            .iter()
            .any(|section| section.words.iter().any(|offered| offered == word))
    }

    /// Whether `word` can be added.
    pub fn gate(&self, word: &str) -> Gate {
        gate_of(self.unavailable.get(word).map(String::as_str))
    }

    fn entry(&self, word: &str, standing: PoolStanding) -> PoolEntry {
        PoolEntry {
            word: word.to_owned(),
            label: word_label(&self.labels, word),
            membership: if self.chosen.iter().any(|held| held == word) {
                PoolMembership::In
            } else {
                PoolMembership::Out
            },
            standing,
        }
    }

    /// The editor's lines: what the module offers, then what the pool holds that it does not.
    pub fn groups(&self) -> Vec<PoolGroup> {
        let offered = self.sections.iter().map(|section| PoolGroup {
            group: section.group.clone(),
            entries: section
                .words
                .iter()
                .map(|word| self.entry(word, PoolStanding::Offered(self.gate(word))))
                .collect(),
        });
        let gone: Vec<PoolEntry> = self
            .chosen
            .iter()
            .filter(|word| !self.offers(word))
            .map(|word| self.entry(word, PoolStanding::Gone))
            .collect();
        let gone = (!gone.is_empty()).then(|| PoolGroup {
            group: Some(GONE_GROUP.to_owned()),
            entries: gone,
        });
        offered.chain(gone).collect()
    }

    /// The models after `word` is turned: added at the end, or taken out.
    pub fn toggled(&self, word: &str) -> Vec<String> {
        if self.chosen.iter().any(|held| held == word) {
            self.chosen
                .iter()
                .filter(|held| *held != word)
                .cloned()
                .collect()
        } else {
            self.chosen
                .iter()
                .cloned()
                .chain(std::iter::once(word.to_owned()))
                .collect()
        }
    }

    /// Whether Automatic has something to choose from: a model in the pool that is offered and
    /// not blocked.
    pub fn readiness(&self) -> ChoiceReadiness {
        let usable = self
            .chosen
            .iter()
            .any(|word| self.offers(word) && !matches!(self.gate(word), Gate::Blocked(_)));
        match (self.chosen.is_empty(), usable) {
            (true, _) => ChoiceReadiness::Missing(ChoiceGap::PoolEmpty),
            (false, false) => ChoiceReadiness::Missing(ChoiceGap::PoolGone),
            (false, true) => ChoiceReadiness::Ready,
        }
    }
}

impl Ready {
    /// The pool at `path`.
    pub(super) fn pool_at(&self, path: &KeyPath) -> Option<&AutoPool> {
        self.kinds
            .iter()
            .flat_map(|kind| kind.rows.iter())
            .map(|row| &row.pool)
            .find(|pool| pool.path.as_ref() == Some(path))
    }

    /// Puts `words` in the pool at `path`, in every row of its slot.
    pub(super) fn set_pool(&mut self, path: &KeyPath, words: &[String]) {
        self.kinds
            .iter_mut()
            .flat_map(|kind| kind.rows.iter_mut())
            .filter(|row| row.pool.path.as_ref() == Some(path))
            .for_each(|row| row.pool.chosen = words.to_vec());
    }
}
