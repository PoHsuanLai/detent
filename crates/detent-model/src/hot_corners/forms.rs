//! What the person types or picks in the forms that follow "Open…" and "Command…", and the
//! binding each makes once it could be one.

use super::binding::{HotDo, HotItem};

/// The kinds of thing "Open…" opens, as its chooser lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HotOpenKind {
    /// An installed app.
    #[default]
    App,
    /// A file.
    File,
    /// A folder.
    Folder,
    /// A web address.
    Web,
}

impl HotOpenKind {
    /// Every kind, in the order the chooser lists them.
    pub const ALL: [HotOpenKind; 4] = [
        HotOpenKind::App,
        HotOpenKind::File,
        HotOpenKind::Folder,
        HotOpenKind::Web,
    ];
}

/// What the "Open…" chooser holds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HotOpenForm {
    /// Which kind is chosen.
    pub kind: HotOpenKind,
    /// The app corner_picked, by its desktop entry's id.
    pub app: String,
    /// The file or folder the file chooser gave.
    pub path: String,
    /// The web address as typed.
    pub web: String,
}

/// The address `typed` stands for, `None` unless it is a web address: `http` or `https`, with a
/// host. A bare `example.org` is taken to be `https://example.org`.
pub fn web_address(typed: &str) -> Option<String> {
    let text = typed.trim();
    if text.is_empty() || text.contains(char::is_whitespace) {
        return None;
    }
    let full = match text.split_once("://") {
        Some((scheme, _)) if scheme == "http" || scheme == "https" => text.to_owned(),
        Some(_) => return None,
        None if text.contains('.') => format!("https://{text}"),
        None => return None,
    };
    let host = full.split_once("://").map(|(_, rest)| rest)?;
    let has_host = host
        .split(['/', '?', '#'])
        .next()
        .is_some_and(|h| !h.is_empty());
    has_host.then_some(full)
}

impl HotOpenForm {
    /// The form that shows `item`, for a corner that already opens it.
    pub fn of_item(item: &HotItem) -> HotOpenForm {
        let blank = HotOpenForm::default();
        match item {
            HotItem::App(id) => HotOpenForm {
                app: id.clone(),
                ..blank
            },
            HotItem::Path(path) => HotOpenForm {
                kind: HotOpenKind::File,
                path: path.clone(),
                ..blank
            },
            HotItem::Web(url) => HotOpenForm {
                kind: HotOpenKind::Web,
                web: url.clone(),
                ..blank
            },
        }
    }

    /// What "Choose" would store, `None` while the form holds nothing to open.
    pub fn item(&self) -> Option<HotItem> {
        let filled = |text: &str| (!text.is_empty()).then(|| text.to_owned());
        match self.kind {
            HotOpenKind::App => filled(&self.app).map(HotItem::App),
            HotOpenKind::File | HotOpenKind::Folder => filled(&self.path).map(HotItem::Path),
            HotOpenKind::Web => web_address(&self.web).map(HotItem::Web),
        }
    }

    /// The binding "Choose" stores.
    pub fn binding(&self) -> Option<HotDo> {
        self.item().map(HotDo::Open)
    }
}

/// The binding a command typed as `typed` makes, `None` while it is blank.
pub fn command_binding(typed: &str) -> Option<HotDo> {
    let command = typed.trim();
    (!command.is_empty()).then(|| HotDo::Command(command.to_owned()))
}
