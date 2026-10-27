//! The Settings app's pure model: which schemas exist, which pages and sections they make, which
//! control draws each key, and what a search finds. No I/O beyond the directory scan in
//! [`discover`], no renderer, no runtime.

mod absence;
mod accounts;
mod advanced;
mod arrival;
mod assistant_model;
mod assistants;
mod bluetooth;
mod catalogue;
mod coding_agents;
mod computers;
mod config_file;
mod control;
mod discovery;
mod displays;
mod engine_install;
mod engines;
mod entry;
mod extensions;
mod first_run;
mod hot_corners;
mod layout;
mod live;
mod memory_consent;
mod navigation;
mod notice;
#[cfg(test)]
mod notice_tests;
mod origin;
mod outside;
mod page;
mod page_kind;
mod picker;
mod picture_add;
mod privacy;
mod range;
mod reason;
mod rows;
mod running;
mod screen_sharing;
mod search;
mod sharing;
mod shortcuts;
mod spaces;
mod style_notes;
mod text_form;
mod updates;
mod wallpaper;
mod warning;
mod wifi;
mod words;
mod work;

pub use absence::Absence;
pub use accounts::{
    ACCOUNTS_SETUP_SLUG, Account, AccountExpiry, AccountKey, AccountPermission, AccountProvider,
    AccountSegment, AccountService, AccountSpend, AccountStatus, AccountSync, AccountTarget,
    AccountsEffect, AccountsInput, AccountsNotice, AccountsPane, AccountsReady, AccountsScreen,
    AccountsState, AddEnd, Adding, Ask, Attention, CalendarDay, Cents, ClientEntry, ClientText,
    ClientValue, DatasetName, GrantDecision, GrantScope, GrantSentence, HoldCounts, HoldEffect,
    HoldInput, Holds, LocalFiles, ModuleId, ModuleRow, Place, ReadTurn, Reads, ReauthEnd,
    RemovalQuestion, Secret, SecretStanding, SignInClient, SignOutEnd, SignedIn, Source, Sources,
    SpendBar, SpendLine, StorageQuota, SwitchState, SyncAdd, SyncBook, SyncHold, SyncNowEnd,
    SyncPause, SyncStatus, SyncWork, accounts_setup_offer, accounts_step, hold_step,
    is_accounts_setup, signed_in,
};
pub use advanced::advanced_name;
pub use arrival::{Arrival, arrival};
pub use assistant_model::{
    ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, SetupAction, SetupAvailability,
    assistant_model_page, assistant_of, is_assistant_setup, setup_availability,
};
pub use assistants::{
    Confirm, ModelOffer, PlaceRef, PlacesEffect, PlacesInput, PlacesState, known_of, offers_for,
    places_step,
};
pub use bluetooth::{
    BLUETOOTH_SLUG, BatteryLevel, BtAddress, BtAsking, BtDevice, BtEffect, BtInput, BtKind, BtLink,
    BtPairing, BtPower, BtReady, BtState, BtStatus, PairingAnswer, PairingAsk, PairingCode,
    PairingQuestion, bluetooth_page, bluetooth_step,
};
pub use catalogue::{Catalogue, FileSpec, PageModel, SectionModel};
pub use coding_agents::{
    AddAction, AddForm, AddRefusal, AgentOffer, AgentReason, AgentStanding, AgentWord, Candidate,
    FoundAgent, Listing, Presence, RemoveAnswer, RemoveQuestion, Request, Step, WORD_PREFIX,
    offers as agent_offers, removal_of, unlisted_found, with_agents,
};
pub use computers::{
    AddList, ComputerRow, ComputersEffect, ComputersFound, ComputersInput, ComputersReady,
    ComputersState, KeyField, NAME_MOST, NewServer, ServerDraft, ServerEdit, ServerFault,
    ServerForm, ServerModel, ServerModels, ServerPhase, ServerReach, ServerRefusal, Standing,
    computers_step, server_models,
};
pub use config_file::ConfigFile;
pub use control::{Control, FileOnly, TogglePair};
pub use discovery::{Discovery, ProgramSchema, discover, discover_under};
pub use displays::{
    ArrangeSide, Brightness, BrightnessList, DISPLAYS_SLUG, DisplayChange, DisplayEdit,
    DisplayMode, DisplayPosition, DisplayRotation, DisplaysEffect, DisplaysInput, DisplaysReady,
    DisplaysState, DisplaysStatus, LooksLike, Millimetres, ModePreference, Monitor, OutputId,
    PixelSize, RefreshMilliHz, ScaleMilli, default_scale, displays_page, displays_step, looks_like,
    mode_at, mode_sizes, native_size, place_beside, side_of,
};
/// Whether an agent may set a key: quire's own type, carried by each `KeySpec`.
pub use ds_settings::schema::AgentSetting;
pub use engine_install::{
    EngineAction, EngineBrief, EngineMatter, EngineOffer, EngineOfferEffect, EngineOfferInput,
    EngineOfferState, EngineRef, EngineRow, EngineStanding, EngineWay, InstallScope, Replacing,
    Waiting, engine_brief, engine_named, engine_offer_step,
};
pub use engines::{
    EnginesEffect, EnginesInput, EnginesState, InstallAdvice, LLAMA_CPP_ID, ListedEngine, LlamaCpp,
    PROGRAMS_SECTION, Program, ProgramRow, Programs, ProgramsEffect, ProgramsInput, ProgramsPane,
    Restart, Restarted, SetupNote, StartStanding, engines_step, install_advice, llama_cpp_of,
    program_rows, programs_step, setup_note,
};
pub use entry::KeyEntry;
pub use extensions::{
    EXTENSIONS_SLUG, ExtensionGroup, ExtensionKeep, ExtensionKind, ExtensionList, ExtensionPower,
    ExtensionRow, ExtensionStanding, ExtensionTrouble, InstalledExtensions, Offer, Offers,
    extensions_page, stray_word,
};
pub use first_run::{
    FirstRun, FirstRunEffect, FirstRunInput, LocalOnly, Runtime, Stage, first_run_step,
};
pub use hot_corners::{
    Corner, CornerAsk, CornerChoice, CornerMenuLine, HOT_CORNERS_CONTROL, HotActionChoice,
    HotActionGroup, HotAddFault, HotApp, HotBinding, HotDesk, HotDo, HotItem, HotKeys, HotKnown,
    HotModifier, HotOpenForm, HotOpenKind, HotPicked, HotShown, action_groups, actions_of,
    actions_of_all, add_row, apps_matching, binding_for, bindings_of, command_binding,
    corner_admits, corner_menu, corner_picked, corner_shown, corner_value, remove_row, shown_rows,
    web_address, with_binding,
};
pub use layout::{Block, advanced_sections, basic_sections, blocks};
pub use live::Live;
pub use memory_consent::{
    Confinement, ConsentState, KeptValue, MEMORY_SECTION, SettingPair, Trouble,
    holds_memory_settings, is_memory_setting, kept_value, tells_confinement, to_confirm,
};
pub use navigation::{landing, opening_page, visible_pages};
pub use notice::{Inbox, Notice, Outbox, Receiver, WindowGone, inbox};
pub use origin::{OriginRule, SchemaOrigin};
pub use outside::{
    AGENTS_OWN, Assistant, Entry, assistants_of, model_choice, model_menu, model_word, way_choice,
    way_menu,
};
pub use page::PageSlug;
pub use page_kind::PageKind;
pub use picker::{
    Applying, AutoPool, AwaitingAccount, Busy, Choice, ChoiceGap, ChoiceReadiness, ChoiceSection,
    Clearing, Described, Gate, KindRows, Loading, LocalAsk, LocalEngine, LocalModel, LocalView,
    MenuLine, ModelRow, PickerEffect, PickerInput, PickerState, PoolEntry, PoolGroup, PoolHeading,
    PoolMembership, PoolStanding, PoolWrite, Ready, TurnOn, TurnOnAnswer, loading_of, step,
};
pub use picture_add::AddOutcome;
pub use privacy::{
    CellKind, DesktopApp, PowerCell, PowerStanding, PrivacyEffect, PrivacyInput, PrivacyPane,
    PrivacyPower, PrivacyProgram, PrivacyReady, PrivacyState, PrivacySwitch, ProgramBook,
    ProgramLook, ProgramSubject, SubjectKind, privacy_step,
};
pub use range::{Number, Range};
pub use reason::Reason;
pub use rows::Columns;
pub use running::{
    InFlight, ModelRun, RunAction, RunBoard, RunEffect, RunInput, RunPane, RunRowView, RunStanding,
    running_step,
};
pub use screen_sharing::{
    APP_ID_MAX, GRANTS_MAX, ScreenGrant, ScreenSharingEffect, ScreenSharingInput,
    ScreenSharingReady, ScreenSharingState, ShareLength, screen_sharing_step,
};
pub use search::{Found, Hit, Rank, Search, search};
pub use sharing::{Existing, SHARING_KEY, Sharing, SharingEffect, SharingInput, Way, sharing_step};
pub use shortcuts::{
    KeymapLayer, KeymapText, KeymapTexts, MachineLayer, Origin, ReplaceQuestion, ShortcutGroup,
    ShortcutProblem, ShortcutRow, ShortcutSection, ShortcutsEffect, ShortcutsInput, ShortcutsState,
    desktop_keymap, replace_question, shortcuts_step,
};
pub use spaces::{
    RemovalAsk, RemovalInput, RemovalStage, SpaceKey, SpaceRow, SpacesEffect, SpacesInput,
    SpacesReady, SpacesState, WithoutMemory, spaces_step,
};
pub use style_notes::{NoteRow, StyleFile, StyleNote, StyleNotes, StyleOwner};
pub use text_form::{TextFault, show_value};
pub use updates::{
    ByteCount, CheckActivity, Checked, Downloaded, NotesState, Permille, ProblemKind, ReleaseNotes,
    UnixSeconds, UpdateAction, UpdateMeter, UpdateOffer, UpdatePhase, UpdateStanding, UpdateStatus,
    UpdateVersion, UpdateWire, UpdatesEffect, UpdatesInput, UpdatesNotice, UpdatesPage,
    UpdatesReady, UpdatesShown, UpdatesState, updates_shown, updates_step,
};
pub use wallpaper::{
    Picture, PictureFrom, PictureMark, PictureSections, PictureTile, current_place, is_picture,
    picture_sections,
};
pub use warning::{StaleReason, Warning};
pub use wifi::{
    AccessPoint, AskKind, AskStanding, HiddenForm, HiddenLogin, HiddenNetwork, HiddenSecurity,
    JoinFailure, NetworkLink, NetworkLock, Passphrase, PasswordAsk, Remembered, SignalBars,
    SignalStrength, Ssid, WIFI_SLUG, WifiAsking, WifiEffect, WifiInput, WifiLogin, WifiNetwork,
    WifiRadio, WifiReady, WifiState, WifiStatus, WifiWire, wifi_page, wifi_step,
};
pub use words::{humanize, word_label};
pub use work::Work;
