//! An action's name in the list and in "Used by".

use chordkit::{Action, StandardAction};

/// Names that read better than the action's slug spelled out.
const WORDED: &[(StandardAction, &str)] = &[
    (StandardAction::NextInputSource, "Next input language"),
    (
        StandardAction::MissionControl,
        "Show all windows and Spaces",
    ),
    (StandardAction::AppWindows, "Show the app's windows"),
    (StandardAction::SpaceLeft, "Move left one Space"),
    (StandardAction::SpaceRight, "Move right one Space"),
    (StandardAction::ScreenshotScreen, "Capture the screen"),
    (StandardAction::ScreenshotSelection, "Capture a selection"),
    (StandardAction::TileLeft, "Tile to the left half"),
    (StandardAction::TileRight, "Tile to the right half"),
    (StandardAction::TileTop, "Tile to the top half"),
    (StandardAction::TileBottom, "Tile to the bottom half"),
    (StandardAction::TileTopLeft, "Tile to the top left quarter"),
    (
        StandardAction::TileTopRight,
        "Tile to the top right quarter",
    ),
    (
        StandardAction::TileBottomLeft,
        "Tile to the bottom left quarter",
    ),
    (
        StandardAction::TileBottomRight,
        "Tile to the bottom right quarter",
    ),
    (StandardAction::Fill, "Fill the screen"),
    (StandardAction::Center, "Center the window"),
    (
        StandardAction::ReturnToPreviousSize,
        "Return to previous size",
    ),
    (StandardAction::LockScreen, "Lock the screen"),
    (StandardAction::Bigger, "Make bigger"),
    (StandardAction::Smaller, "Make smaller"),
    (StandardAction::ActualSize, "Actual size"),
    (StandardAction::ToggleToolbar, "Show or hide the toolbar"),
    (StandardAction::ToggleSidebar, "Show or hide the sidebar"),
    (StandardAction::ToggleDock, "Show or hide the dock"),
];

fn capitalised(text: &str) -> String {
    let mut letters = text.chars();
    letters
        .next()
        .map(|first| first.to_uppercase().chain(letters).collect())
        .unwrap_or_default()
}

fn standard_name(action: StandardAction) -> String {
    if let StandardAction::SwitchToSpace(number) = action {
        return format!("Switch to Space {}", number.get());
    }
    WORDED
        .iter()
        .find(|(known, _)| *known == action)
        .map_or_else(
            || capitalised(&action.label()),
            |(_, name)| (*name).to_owned(),
        )
}

/// The name of `action` in plain words: "Select all", "Switch to Space 3"; an app's action by
/// its id.
pub fn name_of(action: &Action) -> String {
    match action {
        Action::Standard(standard) => standard_name(*standard),
        other => other.id(),
    }
}
