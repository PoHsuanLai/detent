//! The pop-up button at a corner: its lines and what a pick does.

use super::binding::{HotDesk, HotDo};

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
    /// The binding to say in its own words when no line is marked.
    pub says: Option<HotDo>,
}

/// What the pop-up shows for `does`: a desktop action or nothing marks its line; any other
/// binding is said in its own words.
pub fn corner_shown(does: Option<&HotDo>) -> HotShown {
    match does {
        None => HotShown {
            value: Some(CornerChoice::Nothing),
            says: None,
        },
        Some(HotDo::Desk(desk)) => HotShown {
            value: Some(CornerChoice::Desk(*desk)),
            says: None,
        },
        Some(other) => HotShown {
            value: None,
            says: Some(other.clone()),
        },
    }
}
