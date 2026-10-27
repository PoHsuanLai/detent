//! Every sentence the window shows, by pane. The model returns facts (enums); each file here maps
//! them to words, one function per enum, and every other literal the window shows is a constant
//! here, so a change of wording is a change in one file and the model never holds text a person
//! reads.

pub(crate) mod absence;
pub(crate) mod account_notices;
pub(crate) mod account_sync;
pub(crate) mod account_views;
pub(crate) mod accounts;
pub(crate) mod assistant_model;
pub(crate) mod assistants;
pub(crate) mod bluetooth;
pub(crate) mod coding_agents;
pub(crate) mod common;
pub(crate) mod computers;
pub(crate) mod displays;
pub(crate) mod engine_install;
pub(crate) mod engines;
pub(crate) mod extensions;
pub(crate) mod first_run;
pub(crate) mod heroes;
pub(crate) mod hot_corners;
pub(crate) mod memory;
pub(crate) mod notice;
pub(crate) mod picker;
pub(crate) mod pictures;
pub(crate) mod privacy;
pub(crate) mod running;
pub(crate) mod screen_sharing;
pub(crate) mod sharing;
pub(crate) mod shortcuts;
pub(crate) mod spaces;
pub(crate) mod style_notes;
pub(crate) mod updates;
pub(crate) mod wifi;
pub(crate) mod window;
