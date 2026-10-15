//! The Settings app's pure model: which schemas exist, which pages and sections they make, which
//! control draws each key, and what a search finds. No I/O beyond the directory scan in
//! [`discover`], no renderer, no runtime.

mod accounts;
mod advanced;
mod agent;
mod assistant_model;
mod assistants;
mod catalogue;
mod computers;
mod config_file;
mod control;
mod discovery;
mod entry;
mod first_run;
mod layout;
mod navigation;
mod outside;
mod page;
mod picker;
mod range;
mod reads;
mod reason;
mod rows;
mod search;
mod sharing;
mod shortcuts;
mod spaces;
mod text_form;
mod updates;
mod warning;
mod words;

pub use accounts::{
    Account, AccountExpiry, AccountKey, AccountPermission, AccountProvider, AccountSegment,
    AccountService, AccountStatus, AccountSync, AccountTarget, AccountsEffect, AccountsInput,
    AccountsPane, AccountsReady, AccountsScreen, AccountsState, AddEnd, Adding, Ask, ClientEntry,
    DatasetName, HoldCounts, HoldEffect, HoldInput, Holds, KEEP_HELP, LocalFiles, ModuleId,
    ModuleRow, Place, ReadTurn, Reads, ReauthEnd, RemovalQuestion, SignInClient, SignOutEnd,
    Source, Sources, SyncAdd, SyncHold, SyncNowEnd, accounts_step, hold_step, reauth_help,
};
pub use advanced::advanced_name;
pub use agent::AgentSetting;
pub use assistant_model::{
    ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, Pressable, SetupButton, assistant_model_page,
    assistant_of, is_assistant_model, is_assistant_setup, setup_button,
};
pub use assistants::{
    Confirm, ModelOffer, PlaceRef, PlacesEffect, PlacesInput, PlacesState, known_of, offers_for,
    places_step,
};
pub use catalogue::{Catalogue, FileSpec, PageModel, SectionModel};
pub use computers::{
    AddList, ComputerRow, ComputersEffect, ComputersFound, ComputersInput, ComputersReady,
    ComputersState, Standing, computers_step, guest_line,
};
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
pub use outside::{
    AGENTS_OWN, Assistant, Entry, assistants_of, model_choice, model_menu, model_word, way_choice,
    way_menu,
};
pub use page::PageSlug;
pub use picker::{
    Absence, Applying, AwaitingAccount, Choice, ChoiceSection, Described, Gate, KindRows, ModelRow,
    PickerEffect, PickerInput, PickerState, Ready, loading_text, step,
};
pub use range::{Number, Range};
pub use reads::modules_read;
pub use reason::Reason;
pub use rows::{Columns, cell_fits};
pub use search::{Found, Hit, Rank, Search, search};
pub use sharing::{
    ALLOW, CANCEL, Existing, SHARING_KEY, Sharing, SharingEffect, SharingInput, Way,
    sharing_question, sharing_step,
};
pub use shortcuts::{
    KeymapText, KeymapTexts, MachineLayer, Origin, ReplaceQuestion, ShortcutGroup, ShortcutRow,
    ShortcutSection, ShortcutsEffect, ShortcutsInput, ShortcutsState, desktop_keymap, name_of,
    replace_question, shortcuts_step,
};
pub use spaces::{
    HISTORY_CHOICES, HISTORY_NOTE, RemovalSheet, SheetInput, SpaceKey, SpaceRow, SpacesEffect,
    SpacesInput, SpacesReady, SpacesState, spaces_step,
};
pub use text_form::{TextFault, show_value};
pub use updates::{
    ButtonActivity, ByteCount, NotesSheet, NotesState, Permille, UPDATES_NOT_AVAILABLE,
    UnixSeconds, UpdateButton, UpdateMeter, UpdateOffer, UpdatePhase, UpdateStatus, UpdateWire,
    UpdatesEffect, UpdatesInput, UpdatesPage, UpdatesReady, UpdatesShown, UpdatesState,
    is_software_update, updates_shown, updates_step,
};
pub use warning::{StaleReason, Warning};
pub use words::{humanize, word_label};
