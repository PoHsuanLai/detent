//! The memory service's controls, as Settings uses them: the Spaces page settles a removed
//! Space's memories, and a page that holds memory settings asks which looser values wait for the
//! person's word and confirms them, and the Memory section says whether memory keeps to its own
//! folders.

mod drive;
mod view;

pub(crate) use drive::{MemoryConsent, use_memory_consent};
pub(crate) use view::{ConfinementNote, ConsentNote};
