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
mod first_run;
mod hot_corners;
mod layout;
mod live;
mod memory_consent;
mod missing;
mod navigation;
mod notice;
#[cfg(test)]
mod notice_tests;
mod origin;
mod outside;
mod page;
mod picker;
mod picture_add;
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
    AccountsEffect, AccountsInput, AccountsPane, AccountsReady, AccountsScreen, AccountsState,
    AddEnd, Adding, Ask, Attention, Cents, ClientEntry, ClientText, ClientValue, DatasetName,
    GrantDecision, GrantSentence, HoldCounts, HoldEffect, HoldInput, Holds, KEEP_HELP, LocalFiles,
    ModuleId, ModuleRow, Place, ReadTurn, Reads, ReauthEnd, RemovalQuestion, Secret,
    SecretStanding, SignInClient, SignOutEnd, SignedIn, Source, Sources, SpendBar, SpendLine,
    StorageQuota, SyncAdd, SyncBook, SyncHold, SyncNowEnd, SyncPause, SyncStatus, SyncWork,
    accounts_setup_button, accounts_step, hold_step, is_accounts_setup, reauth_help, sign_in_words,
    signed_in,
};
pub use advanced::advanced_name;
pub use arrival::{Arrival, arrival};
pub use assistant_model::{
    ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, Pressable, SetupButton, assistant_model_page,
    assistant_of, is_assistant_model, is_assistant_setup, setup_button,
};
pub use assistants::{
    Confirm, ModelOffer, PlaceRef, PlacesEffect, PlacesInput, PlacesState, known_of, offers_for,
    places_step,
};
pub use bluetooth::{
    BLUETOOTH_SLUG, BatteryLevel, BtAddress, BtDevice, BtEffect, BtInput, BtKind, BtLink,
    BtPairing, BtPower, BtReady, BtSheet, BtState, BtStatus, PairingAnswer, PairingAsk,
    PairingCode, PairingQuestion, bluetooth_page, bluetooth_step, forget_body, forget_title,
    is_bluetooth, missing_words as bluetooth_missing_words, pairing_body, pairing_title,
};
pub use catalogue::{Catalogue, FileSpec, PageModel, SectionModel};
pub use coding_agents::{
    AgentOffer, AgentReason, AgentStanding, CHOOSE_DIRECTLY, GROUP as CODING_AGENTS_GROUP,
    TURNED_OFF, WORD_PREFIX, offers as agent_offers, with_agents,
};
pub use computers::{
    AddList, ComputerRow, ComputersEffect, ComputersFound, ComputersInput, ComputersReady,
    ComputersState, KeyField, NAME_MOST, ServerDraft, ServerEdit, ServerFault, ServerForm,
    ServerModel, ServerModels, ServerPhase, ServerReach, ServerSheet, Standing, computers_step,
    guest_line, server_models,
};
pub use config_file::ConfigFile;
pub use control::{Control, FileOnly, TogglePair};
pub use discovery::{Discovery, ProgramSchema, discover, discover_under};
pub use displays::{
    ArrangeSide, Brightness, BrightnessList, DISPLAYS_SLUG, DisplayChange, DisplayEdit,
    DisplayMode, DisplayPosition, DisplayRotation, DisplaysEffect, DisplaysInput, DisplaysReady,
    DisplaysState, DisplaysStatus, LooksLike, Millimetres, ModePreference, Monitor, OutputId,
    PixelSize, RefreshMilliHz, ScaleMilli, default_scale, displays_page, displays_step,
    hertz_words, is_displays, looks_like, missing_words as displays_missing_words, mode_at,
    mode_sizes, native_size, place_beside, side_of, size_words as display_size_words,
};
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
pub use hot_corners::{
    Corner, CornerAsk, CornerChoice, CornerMenuLine, HOT_CORNERS_CONTROL, HotActionChoice,
    HotActionGroup, HotAddFault, HotApp, HotBinding, HotDesk, HotDo, HotItem, HotKeys, HotKnown,
    HotModifier, HotOpenForm, HotOpenKind, HotPicked, HotShown, action_groups, actions_of,
    actions_of_all, add_row, apps_matching, binding_for, bindings_of, command_binding,
    corner_admits, corner_label, corner_menu, corner_picked, corner_shown, corner_value,
    remove_row, shown_rows, web_address, with_binding,
};
pub use layout::{Block, advanced_sections, basic_sections, blocks};
pub use live::Live;
pub use memory_consent::{
    CONFIRM, Confinement, ConsentState, MEMORY_SECTION, SettingPair, Trouble,
    holds_memory_settings, is_memory_setting, tells_confinement, to_confirm, waiting_note,
};
pub use missing::Missing;
pub use navigation::{landing, opening_page, visible_pages};
pub use notice::{Inbox, Notice, Outbox, Receiver, WindowGone, inbox};
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
pub use picture_add::AddOutcome;
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
pub use style_notes::{NoteRow, StyleFile, StyleNote, StyleNotes, StyleOwner};
pub use text_form::{TextFault, show_value};
pub use updates::{
    ButtonActivity, ByteCount, NotesSheet, NotesState, Permille, ProblemKind,
    UPDATES_NOT_AVAILABLE, UnixSeconds, UpdateButton, UpdateMeter, UpdateOffer, UpdatePhase,
    UpdateStatus, UpdateVersion, UpdateWire, UpdatesEffect, UpdatesInput, UpdatesNotice,
    UpdatesPage, UpdatesReady, UpdatesShown, UpdatesState, is_software_update, updates_shown,
    updates_step,
};
pub use wallpaper::{
    Picture, PictureFrom, PictureMark, PictureSections, PictureTile, current_place, is_picture,
    is_wallpaper, picture_sections,
};
pub use warning::{StaleReason, Warning};
pub use wifi::{
    AccessPoint, AskKind, AskStanding, HiddenForm, HiddenLogin, HiddenNetwork, HiddenSecurity,
    JoinFailure, NetworkLink, NetworkLock, OTHER_FAILED, OTHER_HELP, OTHER_TITLE, Passphrase,
    PasswordAsk, Remembered, SignalBars, SignalStrength, Ssid, WIFI_BLOCKED, WIFI_SLUG, WifiEffect,
    WifiInput, WifiLogin, WifiNetwork, WifiRadio, WifiReady, WifiSheet, WifiState, WifiStatus,
    WifiWire, details_rows, forget_network_body, forget_network_title, is_wifi,
    missing_words as wifi_missing_words, password_help, password_title, wifi_page, wifi_step,
};
pub use words::{STYLE_NOTES_LEAD, STYLE_NOTES_TITLE, humanize, word_label};
pub use work::Work;
