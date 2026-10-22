//! What a corner does, and how a corner's value in the settings file reads as bindings and writes
//! back (hot corners spec, section 1). The file's value is a list of tables, one per set of keys;
//! a file written before the list existed holds one word instead, and still reads.

use super::keys::HotKeys;
use toml::{Table, Value};

/// One of the desktop's own actions, in the order the Mac's pop-up lists them with our extras.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotDesk {
    /// Mission Control: every window.
    Workspaces,
    /// Application Windows: the focused app's windows.
    AppWindows,
    /// Desktop: the windows move aside.
    ShowDesktop,
    /// Notification Centre.
    NotificationCenter,
    /// Control Centre.
    ControlCenter,
    /// The launcher.
    Launcher,
    /// Summon the assistant.
    Assistant,
    /// Lock the screen.
    Lock,
    /// Put the displays to sleep.
    DisplaySleep,
}

impl HotDesk {
    /// Every action, in menu order.
    pub const ALL: [HotDesk; 9] = [
        HotDesk::Workspaces,
        HotDesk::AppWindows,
        HotDesk::ShowDesktop,
        HotDesk::NotificationCenter,
        HotDesk::ControlCenter,
        HotDesk::Launcher,
        HotDesk::Assistant,
        HotDesk::Lock,
        HotDesk::DisplaySleep,
    ];

    /// The actions the pop-up offers, in menu order: every one but putting the displays to
    /// sleep, which the desktop cannot do yet (a file that names it still reads and shows it).
    pub const OFFERED: [HotDesk; 8] = [
        HotDesk::Workspaces,
        HotDesk::AppWindows,
        HotDesk::ShowDesktop,
        HotDesk::NotificationCenter,
        HotDesk::ControlCenter,
        HotDesk::Launcher,
        HotDesk::Assistant,
        HotDesk::Lock,
    ];

    /// The word the file writes.
    pub fn word(self) -> &'static str {
        match self {
            HotDesk::Workspaces => "workspaces",
            HotDesk::AppWindows => "app_windows",
            HotDesk::ShowDesktop => "show_desktop",
            HotDesk::NotificationCenter => "notification_center",
            HotDesk::ControlCenter => "control_center",
            HotDesk::Launcher => "launcher",
            HotDesk::Assistant => "assistant",
            HotDesk::Lock => "lock",
            HotDesk::DisplaySleep => "display_sleep",
        }
    }

    /// What the pop-up calls it.
    pub fn label(self) -> &'static str {
        match self {
            HotDesk::Workspaces => "Mission Control",
            HotDesk::AppWindows => "Application Windows",
            HotDesk::ShowDesktop => "Desktop",
            HotDesk::NotificationCenter => "Notification Centre",
            HotDesk::ControlCenter => "Control Centre",
            HotDesk::Launcher => "Launcher",
            HotDesk::Assistant => "Assistant",
            HotDesk::Lock => "Lock Screen",
            HotDesk::DisplaySleep => "Put Display to Sleep",
        }
    }

    /// The action a file's word names.
    pub fn of_word(word: &str) -> Option<HotDesk> {
        HotDesk::ALL.into_iter().find(|desk| desk.word() == word)
    }
}

/// Something a corner opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotItem {
    /// An installed app, by its desktop entry's id.
    App(String),
    /// A file or a folder; `~` stands for the home folder.
    Path(String),
    /// A web address.
    Web(String),
}

/// What a binding does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotDo {
    /// One of the desktop's own actions.
    Desk(HotDesk),
    /// An action an app declares, with nothing for the person to fill in. `action` is the name
    /// without the app's prefix (`compose`, as the command line takes it).
    AppAction {
        /// The app's name (`org.quire.Mail`).
        app: String,
        /// The action.
        action: String,
    },
    /// Open an app, a file, a folder or a web address.
    Open(HotItem),
    /// Run a command.
    Command(String),
}

/// What a corner does when the pointer rests in it with exactly `keys` held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotBinding {
    /// The keys held.
    pub keys: HotKeys,
    /// What happens.
    pub does: HotDo,
}

/// The text of `table[key]` when it is a non-empty string.
fn text<'a>(table: &'a Table, key: &str) -> Option<&'a str> {
    table
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
}

// SCHEMA: the TOML field names are the spec's: `action`, `keys`, `app`, `intent`, `path`, `url`,
// `command`, with `app_action` and `open` as the action words and `option`, `command`, `control`,
// `shift` as the key words. The list is written as inline tables.
impl HotDo {
    /// The fields a file writes for it, beside the keys.
    fn fields(&self) -> Vec<(&'static str, &str)> {
        match self {
            HotDo::Desk(desk) => vec![("action", desk.word())],
            HotDo::AppAction { app, action } => {
                vec![
                    ("action", "app_action"),
                    ("app", app.as_str()),
                    ("intent", action.as_str()),
                ]
            }
            HotDo::Open(HotItem::App(id)) => vec![("action", "open"), ("app", id.as_str())],
            HotDo::Open(HotItem::Path(path)) => vec![("action", "open"), ("path", path.as_str())],
            HotDo::Open(HotItem::Web(url)) => vec![("action", "open"), ("url", url.as_str())],
            HotDo::Command(command) => vec![("action", "command"), ("command", command.as_str())],
        }
    }

    /// What the table `table` says, `None` when it says nothing this build can carry out. A
    /// command with no text, and the word `none`, say nothing.
    fn of_table(table: &Table) -> Option<HotDo> {
        match text(table, "action")? {
            "app_action" => Some(HotDo::AppAction {
                app: text(table, "app")?.to_owned(),
                action: text(table, "intent")?.to_owned(),
            }),
            "open" => text(table, "app")
                .map(|id| HotItem::App(id.to_owned()))
                .or_else(|| text(table, "path").map(|path| HotItem::Path(path.to_owned())))
                .or_else(|| text(table, "url").map(|url| HotItem::Web(url.to_owned())))
                .map(HotDo::Open),
            "command" => text(table, "command").map(|command| HotDo::Command(command.to_owned())),
            word => HotDesk::of_word(word).map(HotDo::Desk),
        }
    }
}

impl HotBinding {
    /// The binding a table of the list says; `None` for one this build cannot carry out or whose
    /// keys are not modifier words.
    pub fn of_value(value: &Value) -> Option<HotBinding> {
        let table = value.as_table()?;
        let keys = match table.get("keys") {
            None => HotKeys::default(),
            Some(Value::Array(words)) => HotKeys::of_words(
                words
                    .iter()
                    .map(Value::as_str)
                    .collect::<Option<Vec<_>>>()?,
            )?,
            Some(_) => return None,
        };
        Some(HotBinding {
            keys,
            does: HotDo::of_table(table)?,
        })
    }

    /// The table the file holds for it: the keys only when some are held.
    pub fn to_value(&self) -> Value {
        let mut table = Table::new();
        if !self.keys.is_empty() {
            let words = self.keys.words().into_iter();
            table.insert(
                "keys".to_owned(),
                Value::Array(words.map(|word| Value::String(word.to_owned())).collect()),
            );
        }
        for (key, text) in self.does.fields() {
            table.insert(key.to_owned(), Value::String(text.to_owned()));
        }
        Value::Table(table)
    }
}

/// The bindings of a corner's value.
///
/// A list gives the bindings it can read, the first for each set of keys. A word (the form
/// before the list) gives one binding with no keys, or none for `none` and for a word this build
/// does not know. An older `command` word arrives already folded into a list with its command
/// (`ds_settings::schema::fold_older_commands`, when the file is read).
pub fn bindings_of(value: &Value) -> Vec<HotBinding> {
    let read = match value {
        Value::String(word) => {
            let mut table = Table::new();
            table.insert("action".to_owned(), Value::String(word.clone()));
            HotBinding::of_value(&Value::Table(table))
                .into_iter()
                .collect()
        }
        Value::Array(rows) => rows.iter().filter_map(HotBinding::of_value).collect(),
        _ => Vec::new(),
    };
    one_per_keys(read)
}

/// `bindings` without a second binding for a set of keys, ordered by keys (none held first).
pub(super) fn one_per_keys(bindings: Vec<HotBinding>) -> Vec<HotBinding> {
    let mut kept: Vec<HotBinding> = Vec::new();
    for binding in bindings {
        if !kept.iter().any(|have| have.keys == binding.keys) {
            kept.push(binding);
        }
    }
    kept.sort_by(|a, b| a.keys.cmp(&b.keys));
    kept
}

/// The value a corner stores for `bindings`: the list, empty for an inactive corner.
pub fn corner_value(bindings: &[HotBinding]) -> Value {
    Value::Array(bindings.iter().map(HotBinding::to_value).collect())
}

/// Whether a corner's key can hold `value`: a word of the old form this build knows, or a list in
/// which every row is a binding it can carry out. Anything else is kept as written and shown
/// read-only, as every control does with a value it cannot hold.
pub fn corner_admits(value: &Value) -> bool {
    match value {
        Value::String(word) => {
            word == "none" || word == "command" || HotDesk::of_word(word).is_some()
        }
        Value::Array(rows) => rows.iter().all(|row| HotBinding::of_value(row).is_some()),
        _ => false,
    }
}
