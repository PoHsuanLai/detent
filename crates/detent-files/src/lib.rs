//! Reading and writing the settings files the schemas describe (design/22 section 2): a lenient
//! read in which a value no control can hold is kept as written and reported, unknown keys are
//! reported and left alone, and a write that sets one key in place through an atomic
//! temp-and-rename. The programs' own directory watches apply what is written;
//! nothing here talks to a program.

mod agents;
mod error;
mod first_run;
mod keymap;
mod load;
mod places;
mod save;
mod unknown;
mod values;

pub use agents::{load_agents, save_agent_choice};
pub use error::FilesError;
pub use first_run::{FirstRunMark, FirstRunStanding, StateHome};
pub use keymap::{KeymapFiles, KeymapStamp, MachineKeymap};
pub use load::load;
pub use places::{load_places, save_places};
pub use save::save;
pub use values::{ConfigHome, DataHome, FileValues};
