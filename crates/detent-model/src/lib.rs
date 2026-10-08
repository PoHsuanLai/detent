//! The Settings app's pure model: which schemas exist, which pages and sections they make, which
//! control draws each key, and what a search finds. No I/O beyond the directory scan in
//! [`discover`], no renderer, no runtime.

mod accounts;
mod agent;
mod catalogue;
mod config_file;
mod control;
mod discovery;
mod entry;
mod first_run;
mod layout;
mod navigation;
mod page;
mod picker;
mod range;
mod reason;
mod rows;
mod search;
mod text_form;
mod warning;
mod words;

pub use accounts::{
    Account, AccountExpiry, AccountKey, AccountPermission, AccountService, AccountStatus,
    AccountSync, AccountsEffect, AccountsInput, AccountsPane, AccountsReady, AccountsScreen,
    AccountsState, Ask, Place, RemovalQuestion, SignInClient, SignOutEnd, accounts_step,
    reauth_help,
};
pub use agent::AgentSetting;
pub use catalogue::{Catalogue, FileSpec, PageModel, SectionModel};
pub use config_file::ConfigFile;
pub use control::{Control, FileOnly, TogglePair};
pub use discovery::{Discovery, ProgramSchema, discover};
pub use entry::KeyEntry;
pub use first_run::{
    FirstRun, FirstRunEffect, FirstRunInput, LocalOnly, Runtime, Stage, first_run_step,
    runtime_choice,
};
pub use layout::{Block, blocks};
pub use navigation::{landing, visible_pages};
pub use page::PageSlug;
pub use picker::{
    Absence, Applying, ChoiceSection, Described, KindRows, ModelRow, PickerEffect, PickerInput,
    PickerState, Ready, loading_text, step,
};
pub use range::{Number, Range};
pub use reason::Reason;
pub use rows::{Columns, cell_fits};
pub use search::{Found, Hit, Rank, Search, search};
pub use text_form::{TextFault, show_value};
pub use warning::{StaleReason, Warning};
pub use words::{humanize, word_label};
