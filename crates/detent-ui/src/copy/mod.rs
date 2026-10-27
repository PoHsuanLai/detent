//! Every sentence the window shows, by pane. The model returns facts (enums); each file here maps
//! them to words, one function per enum, so a change of wording is a change in one file and the
//! model never holds text a person reads.

mod absence;
pub(crate) mod account_notices;
pub(crate) mod account_sync;
pub(crate) mod accounts;
pub(crate) mod assistants;
pub(crate) mod bluetooth;
pub(crate) mod coding_agents;
mod computers;
pub(crate) mod displays;
pub(crate) mod engine_install;
pub(crate) mod engines;
mod extensions;
pub(crate) mod first_run;
pub(crate) mod hot_corners;
mod memory;
pub(crate) mod picker;
pub(crate) mod pictures;
mod privacy;
pub(crate) mod running;
mod screen_sharing;
pub(crate) mod sharing;
pub(crate) mod shortcuts;
mod spaces;
mod style_notes;
pub(crate) mod updates;
pub(crate) mod wifi;

pub(crate) use absence::{Subject, absence};
pub(crate) use computers::{guest_standing, server_fault, server_refusal};
pub(crate) use extensions::{
    EXTENSIONS_EMPTY_HELP, EXTENSIONS_EMPTY_TITLE, NOT_AVAILABLE, NOT_INSTALLED, extension_line,
    group_title,
};
pub(crate) use memory::{CONFIRM, confinement_line, trouble_line, waiting_note};
pub(crate) use privacy::{SET_BY_ADMINISTRATOR, power_label, program_name, standing_label};
pub(crate) use screen_sharing::grant_line;
pub(crate) use spaces::{
    HISTORY_CHOICES, HISTORY_NOTE, flip_label, history_label, removal_body, removal_title,
};
pub(crate) use style_notes::{STYLE_NOTES_LEAD, STYLE_NOTES_TITLE, note_place, notes_summary};
