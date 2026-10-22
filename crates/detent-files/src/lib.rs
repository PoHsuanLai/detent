//! Reading and writing the settings files the schemas describe (design/22 section 2): a lenient
//! read in which a value no control can hold is kept as written and reported, unknown keys are
//! reported and left alone, and a write that sets one key in place through an atomic
//! temp-and-rename. The programs' own directory watches apply what is written;
//! nothing here talks to a program.

mod agents;
mod coding_agents;
mod desktop_paths;
mod error;
mod first_run;
mod intents;
mod keymap;
mod load;
mod packages;
mod pictures;
mod places;
mod programs;
mod read;
mod save;
mod unknown;
mod values;

pub use agents::{load_agents, save_agent_choice};
pub use coding_agents::{agents_switch, programs_on, turn_on_agents};
pub use error::FilesError;
pub use first_run::{FirstRunMark, FirstRunStanding, StateHome};
pub use intents::read_manifests;
pub use keymap::KeymapFiles;
pub use load::load;
pub use pictures::PictureDirs;
pub use places::{load_places, save_places};
pub use programs::scan_programs;
pub use save::save;
pub use values::{CacheHome, ConfigHome, DataHome, FileValues};
