//! The memory settings that are looser than the shipped ones: memory keeps the shipped value
//! until the person confirms the looser one, and Settings is where they do.

mod confinement;
mod state;

pub use confinement::{Confinement, MEMORY_SECTION, tells_confinement};
pub use state::{
    ConsentState, KeptValue, SettingPair, Trouble, holds_memory_settings, is_memory_setting,
    kept_value, to_confirm,
};
