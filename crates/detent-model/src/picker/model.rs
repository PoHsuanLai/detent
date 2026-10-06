//! What the picker shows: one group per kind, one row per tier.

use crate::words::{humanize, word_label};
use ds_settings::schema::{ChoiceWord, KeyKind, KeyPath, KeySpec, UnavailableReason, WordLabels};
use porter_core::Tier;
use porter_infer::tier_label;
use std::collections::BTreeMap;

/// The word that is Automatic, in the module's rows.
pub(crate) const AUTO: &str = "auto";

/// The prefix of every picker row's path: `ai.model.<slot>.<tier>`.
const PREFIX: &str = "ai.model.";

/// The slots the picker lists, in its order, each with what a person reads. These are
/// porter-infer's `Slot` slugs and labels (inferd's `slot_label`); detent keeps its own copy so it
/// builds against a porter that has no `Slot` yet.
const SLOTS: [(&str, &str); 8] = [
    ("text", "Language"),
    ("voice_in", "Speech to text"),
    ("voice_out", "Text to speech"),
    ("image_in", "Reading images"),
    ("computer_use", "Computer use"),
    ("embeddings", "Embeddings"),
    ("image_gen", "Images"),
    ("rerank", "Reranking"),
];

/// The old kind segments of an inferd that has not moved to slots, and the slot each became.
const LEGACY: [(&str, &str); 3] = [
    ("llm", "text"),
    ("speech_in", "voice_in"),
    ("speech_out", "voice_out"),
];

/// The slot slug a path segment names: itself, or the slot an old kind segment became.
fn slot_of(segment: &str) -> &str {
    LEGACY
        .iter()
        .find(|(old, _)| *old == segment)
        .map_or(segment, |(_, slot)| slot)
}

/// Where a slot sits in the picker; a segment that is no slot goes after every slot.
fn slot_rank(slot: &str) -> usize {
    SLOTS
        .iter()
        .position(|(slug, _)| *slug == slot)
        .unwrap_or(SLOTS.len())
}

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

/// One tier of one slot: a menu of "" (the catalogue's choice), Automatic and each model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelRow {
    /// The module's key, `ai.model.<slot>.<tier>`.
    pub path: KeyPath,
    /// The tier's name as a person reads it ("Demanding" for the Best slot).
    pub tier: &'static str,
    /// The words the menu holds, in the module's order.
    pub words: Vec<String>,
    /// What the module calls each word.
    pub labels: WordLabels,
    /// The words that cannot be picked now, each with the reason shown under it.
    pub unavailable: BTreeMap<String, String>,
    /// The word chosen now: "", `auto` or `account/model`.
    pub value: String,
}

/// The rows of one kind of work.
#[derive(Debug, Clone, PartialEq)]
pub struct KindRows {
    /// The slot's slug, `text` (an old module's `llm` is read as `text`).
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

/// `ai.model.<slot>.<tier>` as its slot (or old kind) segment and tier.
fn parse_path(path: &KeyPath) -> Option<(&str, Tier)> {
    let (kind, tier) = path.0.strip_prefix(PREFIX)?.split_once('.')?;
    let tier = toml::Value::String(tier.to_owned()).try_into().ok()?;
    Some((kind, tier))
}

impl ModelRow {
    /// The row `spec` describes, with `value` chosen; `None` when it is not an
    /// `ai.model.<slot>.<tier>` menu.
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
            unavailable: unavailable_of(&spec.unavailable),
            value,
        };
        Some((kind.to_owned(), row))
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
    /// The picker `described` makes: its menu rows grouped by slot, in the picker's slot order
    /// (rows of a slot the picker does not know follow, in the module's order). A row under an
    /// old kind segment is shown when the module serves no row for the same slot and tier under
    /// the slot's own name, so the page works against an old and a new inferd.
    pub fn of(described: &Described) -> Ready {
        let mut kinds: Vec<KindRows> = Vec::new();
        for spec in &described.keys {
            let value = described
                .values
                .get(&spec.path)
                .cloned()
                .unwrap_or_default();
            let Some((segment, row)) = ModelRow::of(spec, value) else {
                continue;
            };
            let slot = slot_of(&segment).to_owned();
            if segment != slot && shadowed(described, &slot, &spec.path) {
                continue;
            }
            match kinds.iter_mut().find(|rows| rows.kind == slot) {
                Some(rows) => rows.rows.push(row),
                None => kinds.push(KindRows {
                    label: slot_label(&slot)
                        .map_or_else(|| kind_label(&spec.label.0, &slot), str::to_owned),
                    kind: slot,
                    rows: vec![row],
                }),
            }
        }
        kinds.sort_by_key(|rows| slot_rank(&rows.kind));
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

/// What a person reads for a slot the picker knows.
fn slot_label(slot: &str) -> Option<&'static str> {
    SLOTS
        .iter()
        .find(|(slug, _)| *slug == slot)
        .map(|(_, label)| *label)
}

/// Whether the module also serves the row for `legacy`'s tier under the slot's own name.
fn shadowed(described: &Described, slot: &str, legacy: &KeyPath) -> bool {
    let Some((_, tier)) = legacy
        .0
        .strip_prefix(PREFIX)
        .and_then(|rest| rest.split_once('.'))
    else {
        return false;
    };
    let own = format!("{PREFIX}{slot}.{tier}");
    described.keys.iter().any(|spec| spec.path.0 == own)
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
