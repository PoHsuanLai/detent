//! The pop-up button at a corner: its lines, what a pick does, and the words that say what a
//! binding does.

use super::actions::HotKnown;
use super::binding::{HotDesk, HotDo, HotItem};

/// What the person is asked to fill in after a pick that needs more than the pick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerAsk {
    /// An app, a file, a folder or a web address.
    Open,
    /// An action of an installed app.
    AppAction,
    /// The text of a command.
    Command,
}

/// One choice of the pop-up button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerChoice {
    /// One of the desktop's own actions.
    Desk(HotDesk),
    /// Open an item; asks which.
    Open,
    /// Run an app's action; asks which.
    AppAction,
    /// Run a command; asks for it.
    Command,
    /// Nothing happens.
    Nothing,
}

impl CornerChoice {
    /// What the menu calls it. A line that asks for more ends in an ellipsis.
    pub fn label(self) -> &'static str {
        match self {
            CornerChoice::Desk(desk) => desk.label(),
            CornerChoice::Open => "Open\u{2026}",
            CornerChoice::AppAction => "App Action\u{2026}",
            CornerChoice::Command => "Command\u{2026}",
            CornerChoice::Nothing => "\u{2013}",
        }
    }
}

/// A line of the pop-up's menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerMenuLine {
    /// A choice.
    Choice(CornerChoice),
    /// A rule between groups.
    Separator,
}

/// The lines of every corner's pop-up: the desktop's actions, then Open and App Action, then
/// Command, then nothing, a rule between the groups.
pub fn corner_menu() -> Vec<CornerMenuLine> {
    let desk = HotDesk::OFFERED
        .into_iter()
        .map(|desk| CornerMenuLine::Choice(CornerChoice::Desk(desk)));
    let groups: [&[CornerChoice]; 3] = [
        &[CornerChoice::Open, CornerChoice::AppAction],
        &[CornerChoice::Command],
        &[CornerChoice::Nothing],
    ];
    desk.chain(groups.into_iter().flat_map(|group| {
        std::iter::once(CornerMenuLine::Separator)
            .chain(group.iter().map(|choice| CornerMenuLine::Choice(*choice)))
    }))
    .collect()
}

/// What a pick comes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HotPicked {
    /// The corner's binding for these keys becomes this, or goes away.
    Set(Option<HotDo>),
    /// More is needed before it can.
    Ask(CornerAsk),
}

/// What picking `choice` comes to.
pub fn corner_picked(choice: CornerChoice) -> HotPicked {
    match choice {
        CornerChoice::Desk(desk) => HotPicked::Set(Some(HotDo::Desk(desk))),
        CornerChoice::Nothing => HotPicked::Set(None),
        CornerChoice::Open => HotPicked::Ask(CornerAsk::Open),
        CornerChoice::AppAction => HotPicked::Ask(CornerAsk::AppAction),
        CornerChoice::Command => HotPicked::Ask(CornerAsk::Command),
    }
}

/// What a pop-up button shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotShown {
    /// The line of the menu that is marked, if one is.
    pub value: Option<CornerChoice>,
    /// The button's words when no line is marked: what the binding does.
    pub title: Option<String>,
}

/// What the pop-up shows for `does`: a desktop action or nothing marks its line; any other
/// binding says what it does in its own words.
pub fn corner_shown(does: Option<&HotDo>, known: &HotKnown) -> HotShown {
    match does {
        None => HotShown {
            value: Some(CornerChoice::Nothing),
            title: None,
        },
        Some(HotDo::Desk(desk)) => HotShown {
            value: Some(CornerChoice::Desk(*desk)),
            title: None,
        },
        Some(other) => HotShown {
            value: None,
            title: Some(corner_label(other, known)),
        },
    }
}

/// The last name in `path`, without a trailing slash; the path itself when it has none.
fn name_of_path(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
}

/// The host of a web address, without its scheme, path or port.
fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split(['/', '?', '#'])
        .next()
        .and_then(|authority| authority.rsplit('@').next())
        .and_then(|host| host.split(':').next())
        .filter(|host| !host.is_empty())
        .unwrap_or(url)
}

/// The longest command a pop-up button spells out.
const COMMAND_SHOWN: usize = 28;

/// `text` cut to `COMMAND_SHOWN` characters, with an ellipsis when it was cut.
fn short(text: &str) -> String {
    let line = text.lines().next().unwrap_or_default().trim();
    match line.char_indices().nth(COMMAND_SHOWN) {
        Some((at, _)) => format!("{}\u{2026}", &line[..at]),
        None => line.to_owned(),
    }
}

/// What `does` is called where a person reads it: "Open Files", "Mail › Compose",
/// "Command: notify-send hi".
pub fn corner_label(does: &HotDo, known: &HotKnown) -> String {
    match does {
        HotDo::Desk(desk) => desk.label().to_owned(),
        HotDo::AppAction { app, action } => known.action_line(app, action),
        HotDo::Open(HotItem::App(id)) => format!("Open {}", known.app_title(id)),
        HotDo::Open(HotItem::Path(path)) => format!("Open {}", name_of_path(path)),
        HotDo::Open(HotItem::Web(url)) => format!("Open {}", host_of(url)),
        HotDo::Command(command) => format!("Command: {}", short(command)),
    }
}
