//! Whether memory keeps itself to its own folders.

use super::state::is_memory_setting;
use crate::catalogue::SectionModel;

/// The Memory section of the Intelligence page, the one the line sits under.
pub const MEMORY_SECTION: &str = "Memory";

/// How far memory confines itself, as it names it on the bus (the `Confinement` property). The
/// words are memory's public contract. Hand-written rather than a `Word`: a word this build does
/// not know is a value too, and memory not being there is the same value, since neither gets a
/// line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Confinement {
    /// Every restriction is in force.
    Confined,
    /// The computer's system is older than the policy: part of it is in force.
    Partial,
    /// The computer's system has nothing to confine memory with.
    NotAvailable,
    /// A test build that switched confinement off. Never shown.
    Off,
    /// A word this build does not know, or memory did not answer. Never shown.
    #[default]
    Unknown,
}

impl Confinement {
    /// The value memory's `word` names; any other word is [`Confinement::Unknown`].
    pub fn from_word(word: &str) -> Confinement {
        match word {
            "confined" => Confinement::Confined,
            "partial" => Confinement::Partial,
            "not-available" => Confinement::NotAvailable,
            "off" => Confinement::Off,
            _ => Confinement::Unknown,
        }
    }
}

/// Whether `section` is the one that tells the person how far memory confines itself: the
/// Memory section, when it holds memory's settings.
pub fn tells_confinement(section: &SectionModel) -> bool {
    section.title == MEMORY_SECTION && section.keys.iter().any(is_memory_setting)
}
