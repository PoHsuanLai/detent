//! What the picker shows: one group per kind, one row per tier.

use crate::words::word_label;
use ds_settings::schema::{ChoiceWord, KeyKind, KeyPath, KeySpec, UnavailableReason, WordLabels};
use std::collections::BTreeMap;

/// The word that is Automatic, in the module's rows.
pub(crate) const AUTO: &str = "auto";

/// The prefix of every picker row's path: `ai.model.<slot>.<tier>`.
const PREFIX: &str = "ai.model.";

/// What separates the area from the slot in a row's section: `Models / Text`.
const SECTION_SPLIT: &str = " / ";

/// Why a live pane (the model picker, the Accounts pane) is not there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Absence {
    /// The window was started without a bus to look on.
    NoBus,
    /// Nothing serves the module: the daemon is not running.
    NotRunning,
    /// The daemon refused this caller: it does not list Settings in its callers.
    NotPermitted,
    /// Anything else the module or the bus said.
    Failed(String),
}

impl std::fmt::Display for Absence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Absence::NoBus => f.write_str("Settings is not connected to the desktop"),
            Absence::NotRunning => f.write_str("the service is not running"),
            Absence::NotPermitted => f.write_str("Settings was not opened from the desktop"),
            Absence::Failed(_) => f.write_str("the service did not answer"),
        }
    }
}

/// One run of a row's choices: the ungrouped ones, or one group's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceSection {
    /// The group's display name; `None` for the ungrouped choices at the top.
    pub group: Option<String>,
    /// The stored words, in the module's order.
    pub words: Vec<String>,
}

/// One tier of one slot: a menu of "" (the catalogue's choice), Automatic and each model.
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
    /// The word chosen now: "", `auto` or `account/model`.
    pub value: String,
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
#[derive(Debug, Clone, PartialEq)]
pub struct Described {
    /// The module's keys.
    pub keys: Vec<KeySpec>,
    /// The value of each key the module answered for.
    pub values: BTreeMap<KeyPath, String>,
}

/// A set that is in flight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applying {
    /// The row being set.
    pub path: KeyPath,
    /// The word it is being set to.
    pub word: String,
}

/// A picker that is there.
#[derive(Debug, Clone, PartialEq)]
pub struct Ready {
    /// The slots, in the order their rows first appear in `Describe`.
    pub kinds: Vec<KindRows>,
    /// The set in flight, if any: one at a time.
    pub applying: Option<Applying>,
}

/// Where the picker stands.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PickerState {
    /// Looking for the module.
    #[default]
    Probing,
    /// The module is not there, and why.
    Unavailable(Absence),
    /// The module answered.
    Ready(Ready),
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
            sections: spec
                .grouped_choices()
                .into_iter()
                .map(|section| ChoiceSection {
                    group: section.group.map(|group| group.0.clone()),
                    words: section.choices.into_iter().map(str::to_owned).collect(),
                })
                .collect(),
            labels: spec.labels.clone(),
            unavailable: unavailable_of(&spec.unavailable),
            value,
        })
    }

    /// The reason `word` cannot be picked, if it cannot.
    pub fn unavailable_reason(&self, word: &str) -> Option<&str> {
        self.unavailable.get(word).map(String::as_str)
    }

    /// What the menu calls `word`.
    pub fn label_of(&self, word: &str) -> String {
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
        Ready {
            kinds,
            applying: None,
        }
    }

    /// The row at `path`.
    pub(crate) fn row_mut(&mut self, path: &KeyPath) -> Option<&mut ModelRow> {
        self.kinds
            .iter_mut()
            .flat_map(|kind| kind.rows.iter_mut())
            .find(|row| &row.path == path)
    }
}

/// The module's unavailable choices as plain words and reasons.
fn unavailable_of(
    unavailable: &BTreeMap<ChoiceWord, UnavailableReason>,
) -> BTreeMap<String, String> {
    unavailable
        .iter()
        .map(|(word, reason)| (word.0.clone(), reason.0.clone()))
        .collect()
}

/// What a row shows while `word` is being set: "Loading <model>…" for a model, "Applying" for
/// the two words that name no model.
pub fn loading_text(row: &ModelRow, word: &str) -> String {
    if word.is_empty() || word == AUTO {
        "Applying".to_owned()
    } else {
        format!("Loading {}…", row.label_of(word))
    }
}
