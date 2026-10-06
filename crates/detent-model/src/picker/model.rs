//! What the picker shows: one group per kind, one row per tier.

use crate::words::{humanize, word_label};
use ds_settings::schema::{KeyKind, KeyPath, KeySpec, WordLabels};
use porter_core::Tier;
use porter_infer::tier_label;
use std::collections::BTreeMap;

/// The word that is Automatic, in the module's rows.
pub(crate) const AUTO: &str = "auto";

/// The prefix of every picker row's path: `ai.model.<kind>.<tier>`.
const PREFIX: &str = "ai.model.";

/// Why the picker is not there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Absence {
    /// The window was started without a bus to look on.
    NoBus,
    /// Nothing serves the module: inferd is not running.
    NotRunning,
    /// inferd refused this caller: it does not list Settings in its callers.
    NotPermitted,
    /// Anything else the module or the bus said.
    Failed(String),
}

/// One tier of one kind: a menu of "" (the catalogue's choice), Automatic and each model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelRow {
    /// The module's key, `ai.model.<kind>.<tier>`.
    pub path: KeyPath,
    /// The tier's name as a person reads it ("Demanding" for the Best slot).
    pub tier: &'static str,
    /// The words the menu holds, in the module's order.
    pub words: Vec<String>,
    /// What the module calls each word.
    pub labels: WordLabels,
    /// The word chosen now: "", `auto` or `account/model`.
    pub value: String,
}

/// The rows of one kind of work.
#[derive(Debug, Clone, PartialEq)]
pub struct KindRows {
    /// The kind's slug, `llm`.
    pub kind: String,
    /// What a person reads, "Language".
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
    /// The kinds, in the order the module lists them.
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

/// `ai.model.<kind>.<tier>` as its kind slug and tier.
fn parse_path(path: &KeyPath) -> Option<(&str, Tier)> {
    let (kind, tier) = path.0.strip_prefix(PREFIX)?.split_once('.')?;
    let tier = toml::Value::String(tier.to_owned()).try_into().ok()?;
    Some((kind, tier))
}

impl ModelRow {
    /// The row `spec` describes, with `value` chosen; `None` when it is not an
    /// `ai.model.<kind>.<tier>` menu.
    fn of(spec: &KeySpec, value: String) -> Option<(String, ModelRow)> {
        let (kind, tier) = parse_path(&spec.path)?;
        let KeyKind::Menu { variants } = &spec.kind else {
            return None;
        };
        let row = ModelRow {
            path: spec.path.clone(),
            tier: tier_label(tier),
            words: variants.clone(),
            labels: spec.labels.clone(),
            value,
        };
        Some((kind.to_owned(), row))
    }

    /// What the menu calls `word`.
    pub fn label_of(&self, word: &str) -> String {
        word_label(&self.labels, word)
    }
}

impl Ready {
    /// The picker `described` makes: its menu rows grouped by kind, in the module's order.
    pub fn of(described: &Described) -> Ready {
        let mut kinds: Vec<KindRows> = Vec::new();
        for spec in &described.keys {
            let value = described
                .values
                .get(&spec.path)
                .cloned()
                .unwrap_or_default();
            let Some((kind, row)) = ModelRow::of(spec, value) else {
                continue;
            };
            match kinds.iter_mut().find(|rows| rows.kind == kind) {
                Some(rows) => rows.rows.push(row),
                None => kinds.push(KindRows {
                    label: kind_label(&spec.label.0, &kind),
                    kind,
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

/// The kind as a person reads it: the part of the row's label before the tier ("Language: Fast"),
/// else the slug humanized.
fn kind_label(label: &str, kind: &str) -> String {
    label
        .split_once(": ")
        .map_or_else(|| humanize(kind), |(name, _)| name.to_owned())
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
