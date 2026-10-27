//! What waits for the person's word, and the pairs that are confirmed by saving them.

use super::confinement::Confinement;
use crate::catalogue::PageModel;
use crate::control::Control;
use crate::entry::KeyEntry;
use crate::text_form::show_value;
use crate::words::word_label;
use almanac_core::WaitingSetting;
use std::collections::BTreeMap;
use std::path::Path;

/// The settings file whose looser values wait for confirmation (almanac's, under the config
/// home).
const MEMORY_FILE: &str = "almanac/settings.toml";

/// Whether `entry` is a memory setting, one that can wait for the person's confirmation.
pub fn is_memory_setting(entry: &KeyEntry) -> bool {
    entry.file.relative() == Path::new(MEMORY_FILE)
}

/// Whether any key of `page`, basic or advanced, is a memory setting.
pub fn holds_memory_settings(page: &PageModel) -> bool {
    page.sections
        .iter()
        .chain(&page.advanced)
        .flat_map(|section| &section.keys)
        .any(is_memory_setting)
}

/// A value of a memory setting, as the row names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeptValue {
    /// A number with its unit, as the row's slider shows it (`30 days`).
    Measured(String),
    /// A word, by its label.
    Named(String),
    /// Anything else, as memory wrote it.
    Written(String),
}

/// The value memory keeps while `setting` waits, the way the row of `entry` reads it.
pub fn kept_value(entry: &KeyEntry, setting: &WaitingSetting) -> KeptValue {
    let wire = setting.shipped.as_str();
    let value = entry.control.from_text(wire).ok();
    match (&entry.control, value) {
        (Control::Slider(range), Some(value)) => KeptValue::Measured(range.display(&value)),
        (Control::Toggle(_) | Control::Segmented(_) | Control::Menu(_), Some(_)) => {
            KeptValue::Named(word_label(&entry.spec.labels, wire))
        }
        _ => KeptValue::Written(wire.to_owned()),
    }
}

/// A setting key and one value for it, as the memory service names them: numbers in decimal,
/// words as the file has them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SettingPair {
    /// The dotted key, like `memory.retention.search_days`.
    pub key: String,
    /// Its value as text.
    pub value: String,
}

impl SettingPair {
    /// The pair the person made by setting `entry` to `value`.
    pub fn of(entry: &KeyEntry, value: &toml::Value) -> SettingPair {
        SettingPair {
            key: entry.spec.path.0.clone(),
            value: show_value(value),
        }
    }
}

/// Why a confirmation did not go through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trouble {
    /// Memory would not take the confirmation from this window.
    NotAllowed,
    /// Memory did not accept the value.
    Invalid,
    /// Anything else: the call did not come back right.
    Failed,
}

/// The pairs of `set` that memory reports as waiting with exactly that value: the ones the person
/// just set that were looser than the shipped value, so saving them is the confirmation. A pair
/// memory does not report waiting is not looser (or is already confirmed) and is left out.
pub fn to_confirm(waiting: &[WaitingSetting], set: &[SettingPair]) -> Vec<SettingPair> {
    set.iter()
        .filter(|pair| {
            waiting
                .iter()
                .any(|held| held.key == pair.key && held.waiting == pair.value)
        })
        .cloned()
        .collect()
}

/// The loose values waiting for the person, the keys whose last confirmation failed, and how far
/// memory confines itself.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsentState {
    waiting: Vec<WaitingSetting>,
    troubles: BTreeMap<String, Trouble>,
    confinement: Confinement,
}

impl ConsentState {
    /// The state after memory reported `waiting`. A failure stays only while its key waits.
    #[must_use]
    pub fn listed(self, waiting: Vec<WaitingSetting>) -> ConsentState {
        let troubles = self
            .troubles
            .into_iter()
            .filter(|(key, _)| waiting.iter().any(|held| &held.key == key))
            .collect();
        ConsentState {
            waiting,
            troubles,
            ..self
        }
    }

    /// The state after confirming `keys` failed with `trouble`.
    #[must_use]
    pub fn troubled(self, keys: &[String], trouble: Trouble) -> ConsentState {
        let mut troubles = self.troubles;
        troubles.extend(keys.iter().map(|key| (key.clone(), trouble)));
        ConsentState { troubles, ..self }
    }

    /// The state after confirming `keys` went through: their failures are over.
    #[must_use]
    pub fn cleared(self, keys: &[String]) -> ConsentState {
        let mut troubles = self.troubles;
        troubles.retain(|key, _| !keys.contains(key));
        ConsentState { troubles, ..self }
    }

    /// The state after memory said how far it confines itself.
    #[must_use]
    pub fn confined(self, confinement: Confinement) -> ConsentState {
        ConsentState {
            confinement,
            ..self
        }
    }

    /// How far memory confines itself, as of the last read; unknown before one.
    pub fn confinement(&self) -> Confinement {
        self.confinement
    }

    /// The value of `key` that waits, if one does.
    pub fn waiting_for(&self, key: &str) -> Option<&WaitingSetting> {
        self.waiting.iter().find(|held| held.key == key)
    }

    /// Why the last confirmation of `key` failed, if it did.
    pub fn trouble_for(&self, key: &str) -> Option<Trouble> {
        self.troubles.get(key).copied()
    }
}
