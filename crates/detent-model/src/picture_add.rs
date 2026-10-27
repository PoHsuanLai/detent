//! How adding a picture ended.

use std::path::PathBuf;

/// How asking for a picture and keeping it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddOutcome {
    /// The picture is kept: where it lies now, the value the key stores.
    Added(PathBuf),
    /// The person closed the chooser: nothing is said.
    Closed,
    /// The chosen file (by name) is not a kind of picture the desktop draws.
    NotAPicture(String),
    /// The chosen picture (by name) could not be copied among the added ones.
    CouldNotCopy(String),
    /// There is no window to choose a picture in.
    NoChooser,
}
