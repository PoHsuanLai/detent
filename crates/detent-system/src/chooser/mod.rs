//! A folder or a picture chosen by the person in the desktop's own chooser (the file chooser
//! portal), never typed: the one way a Strata they already have is pointed at, and the one way a
//! wallpaper is added.

mod bus;

pub use bus::OnPortal;

use detent_model::Work;
use std::fmt::Debug;

/// What the person is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Want {
    /// A folder.
    Folder,
    /// Any file.
    File,
    /// A picture file.
    Picture,
}

/// What asking the person came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asked {
    /// They chose a file on this computer: its path.
    Path(String),
    /// They closed the chooser, or chose nothing that is a file here.
    Closed,
    /// There is no chooser to ask.
    Unavailable,
}

/// The desktop's chooser.
pub trait ChooserLink: Debug + Send + Sync {
    /// Asks the person for a folder, a file or a picture, and waits for the answer.
    fn ask(&self, want: Want) -> Work<'_, Asked>;
}

/// Asks the person for a folder, a file or a picture, and waits for the answer; `None` when the
/// chooser is closed, there is none, or what was chosen is not a file on this computer.
pub async fn choose(chooser: &dyn ChooserLink, want: Want) -> Option<String> {
    match chooser.ask(want).await {
        Asked::Path(path) => Some(path),
        Asked::Closed | Asked::Unavailable => None,
    }
}
