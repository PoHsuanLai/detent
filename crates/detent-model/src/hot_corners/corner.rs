//! The four corners and the keys of the settings file that hold them.

// The marker is quire's control name `corners` (`Control::Corners`) on the four corner keys.
/// The control name a schema gives the corner keys to have the sheet draw them.
pub const HOT_CORNERS_CONTROL: &str = "corners";

/// A corner of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Corner {
    /// Top left.
    TopLeft,
    /// Top right.
    TopRight,
    /// Bottom left.
    BottomLeft,
    /// Bottom right.
    BottomRight,
}

impl Corner {
    /// Every corner, row by row from the top.
    pub const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];

    // SCHEMA: the corner keys are `hot_corners.<corner>` and hold a list of tables.
    /// The key that holds the corner's bindings.
    pub fn path(self) -> &'static str {
        match self {
            Corner::TopLeft => "hot_corners.top_left",
            Corner::TopRight => "hot_corners.top_right",
            Corner::BottomLeft => "hot_corners.bottom_left",
            Corner::BottomRight => "hot_corners.bottom_right",
        }
    }

    /// What names the corner to a person who cannot see the sheet's picture.
    pub fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }

    /// The corner whose bindings the key `path` holds.
    pub fn of_path(path: &str) -> Option<Corner> {
        Corner::ALL.into_iter().find(|corner| corner.path() == path)
    }
}
