//! The Settings app's pure model: which schemas exist, which pages and sections they make, which
//! control draws each key, and what a search finds. No I/O beyond the directory scan in
//! [`discover`], no renderer, no runtime.

mod absence;
mod accounts;
mod advanced;
mod assistant_model;
mod assistants;
mod catalogue;
mod coding_agents;
mod computers;
mod config_file;
mod control;
mod discovery;
mod engine_install;
mod engines;
mod entry;
mod first_run;
mod layout;
mod live;
mod memory_consent;
mod navigation;
mod origin;
mod outside;
mod page;
mod picker;
mod privacy;
mod range;
mod reads;
mod reason;
mod rows;
mod screen_sharing;
mod search;
mod sharing;
mod shortcuts;
mod spaces;
mod text_form;
mod updates;
mod warning;
mod words;

pub use absence::Absence;
pub use accounts::{
    Account, AccountExpiry, AccountKey, AccountPermission, AccountProvider, AccountSegment,
    AccountService, AccountStatus, AccountSync, AccountTarget, AccountsEffect, AccountsInput,
    AccountsPane, AccountsReady, AccountsScreen, AccountsState, AddEnd, Adding, Ask, ClientEntry,
    ClientText, ClientValue, DatasetName, HoldCounts, HoldEffect, HoldInput, Holds, KEEP_HELP,
    LocalFiles, ModuleId, ModuleRow, Place, ReadTurn, Reads, ReauthEnd, RemovalQuestion, Secret,
    SecretStanding, SignInClient, SignOutEnd, Source, Sources, SyncAdd, SyncHold, SyncNowEnd,
    accounts_step, hold_step, reauth_help,
};
pub use advanced::advanced_name;
pub use assistant_model::{
    ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, Pressable, SetupButton, assistant_model_page,
    assistant_of, is_assistant_model, is_assistant_setup, setup_button,
};
pub use assistants::{
    Confirm, ModelOffer, PlaceRef, PlacesEffect, PlacesInput, PlacesState, known_of, offers_for,
    places_step,
};
pub use catalogue::{Catalogue, FileSpec, PageModel, SectionModel};
pub use coding_agents::{
    AgentOffer, AgentStanding, CHOOSE_DIRECTLY, GROUP as CODING_AGENTS_GROUP,
    Reason as AgentReason, TURNED_OFF, WORD_PREFIX, offers as agent_offers, with_agents,
};
pub use computers::{
    AddList, ComputerRow, ComputersEffect, ComputersFound, ComputersInput, ComputersReady,
    ComputersState, Standing, computers_step, guest_line,
};
pub use config_file::ConfigFile;
pub use control::{Control, FileOnly, TogglePair};
pub use discovery::{Discovery, ProgramSchema, discover, discover_under};
/// Whether an agent may set a key: quire's own type, carried by each `KeySpec`.
pub use ds_settings::schema::AgentSetting;
pub use engine_install::{
    EngineRef, EngineRow, EngineSheet, EngineSheetEffect, EngineSheetInput, EngineSheetState,
    EngineStanding, EngineWay, INSTALL_FOR, InstallScope, Press, Replacing, SheetButton,
    SheetWords, Waiting, engine_named, engine_sheet_step, sheet_words, size_words,
};
pub use engines::{
    EnginesEffect, EnginesInput, EnginesState, InstallAdvice, LLAMA_CPP_ID, ListedEngine, LlamaCpp,
    Program, RELEASES_LINE, RESTART, Restart, Restarted, SETUP_LINE, START_USING, SetupNote,
    StartButton, engines_step, install_advice, llama_cpp_of, restart_question, setup_note,
};
pub use entry::KeyEntry;
pub use first_run::{
    FirstRun, FirstRunEffect, FirstRunInput, LocalOnly, Runtime, Stage, first_run_step,
};
pub use layout::{Block, blocks};
pub use live::Live;
pub use memory_consent::{
    CONFIRM, Confinement, ConsentState, MEMORY_SECTION, SettingPair, Trouble,
    holds_memory_settings, is_memory_setting, tells_confinement, to_confirm, waiting_note,
};
pub use navigation::{landing, visible_pages};
pub use origin::{OriginRule, SchemaOrigin};
pub use outside::{
    AGENTS_OWN, Assistant, Entry, assistants_of, model_choice, model_menu, model_word, way_choice,
    way_menu,
};
pub use page::PageSlug;
pub use picker::{
    Applying, AutoPool, AwaitingAccount, Busy, Choice, ChoiceGap, ChoiceReadiness, ChoiceSection,
    Clearing, Described, Gate, KindRows, LocalAsk, LocalEngine, LocalModel, LocalView, MenuLine,
    ModelRow, PickerEffect, PickerInput, PickerState, PoolEntry, PoolGroup, PoolMembership,
    PoolStanding, PoolWrite, Ready, TurnOn, TurnOnAnswer, loading_text, step,
};
pub use privacy::{
    CellKind, DesktopApp, PowerCell, PowerStanding, PrivacyEffect, PrivacyInput, PrivacyPane,
    PrivacyPower, PrivacyProgram, PrivacyReady, PrivacyState, PrivacySwitch, ProgramBook,
    ProgramLook, ProgramSubject, SET_BY_ADMINISTRATOR, SubjectKind, privacy_step,
};
pub use range::{Number, Range};
pub use reads::modules_read;
pub use reason::Reason;
pub use rows::Columns;
pub use screen_sharing::{
    APP_ID_MAX, GRANTS_MAX, ScreenGrant, ScreenSharingEffect, ScreenSharingInput,
    ScreenSharingReady, ScreenSharingState, ShareLength, grant_line, screen_sharing_step,
};
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
    HISTORY_CHOICES, HISTORY_NOTE, MEMORY_NOT_RUNNING, RemovalSheet, RemovalStage, SheetInput,
    SpaceKey, SpaceRow, SpacesEffect, SpacesInput, SpacesReady, SpacesState, WithoutMemory,
    spaces_step,
};
pub use text_form::{TextFault, show_value};
pub use updates::{
    ButtonActivity, ByteCount, NotesSheet, NotesState, Permille, ProblemKind,
    UPDATES_NOT_AVAILABLE, UnixSeconds, UpdateButton, UpdateMeter, UpdateOffer, UpdatePhase,
    UpdateStatus, UpdateVersion, UpdateWire, UpdatesEffect, UpdatesInput, UpdatesNotice,
    UpdatesPage, UpdatesReady, UpdatesShown, UpdatesState, is_software_update, updates_shown,
    updates_step,
};
pub use warning::{StaleReason, Warning};
pub use words::{humanize, word_label};
