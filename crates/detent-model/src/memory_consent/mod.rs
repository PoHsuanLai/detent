//! The memory settings that are looser than the shipped ones: memory keeps the shipped value
//! until the person confirms the looser one, and Settings is where they do.

mod confinement;
mod state;
mod words;

pub use confinement::{Confinement, MEMORY_SECTION, tells_confinement};
pub use state::{ConsentState, SettingPair, Trouble, is_memory_setting, to_confirm};
pub use words::{CONFIRM, holds_memory_settings, waiting_note};
