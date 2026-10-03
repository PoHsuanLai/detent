//! The Settings app's pure model: which schemas exist, which pages and sections they make, which
//! control draws each key, and what a search finds. No I/O beyond the directory scan in
//! [`discover`], no renderer, no runtime.

mod agent;
mod catalogue;
mod config_file;
mod control;
mod discovery;
mod entry;
mod navigation;
mod page;
mod range;
mod search;
mod text_form;
mod warning;
mod words;

pub use agent::AgentSetting;
pub use catalogue::{Catalogue, FileSpec, PageModel, SectionModel};
pub use config_file::ConfigFile;
pub use control::{Control, FileOnly, TogglePair};
pub use discovery::{Discovery, ProgramSchema, discover};
pub use entry::KeyEntry;
pub use navigation::{landing, visible_pages};
pub use page::PageSlug;
pub use range::{Number, Range};
pub use search::{Hit, Rank, Search, search};
pub use text_form::{TextFault, show_value};
pub use warning::{StaleReason, Warning};
pub use words::{humanize, word_label};
