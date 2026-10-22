//! What the Hot Corners sheet asks of the computer: the actions installed apps offer, and a file
//! or a folder chosen by the person.

use detent_model::{HotActionChoice, HotOpenKind, Work};
use std::fmt::Debug;

/// The things the sheet asks for.
pub trait CornerDesk: Debug + Send + Sync {
    /// Every action of the installed apps that a corner can run as it is.
    fn actions(&self) -> Work<'_, Vec<HotActionChoice>>;
    /// Asks the person for a file or a folder (`kind` says which; any other kind asks nothing)
    /// and waits: the path chosen, or `None` when they closed the chooser or there is none.
    fn choose(&self, kind: HotOpenKind) -> Work<'_, Option<String>>;
}

/// No apps with actions and no chooser.
#[derive(Debug, Clone, Copy)]
pub struct NoCorners;

impl CornerDesk for NoCorners {
    fn actions(&self) -> Work<'_, Vec<HotActionChoice>> {
        Box::pin(async { Vec::new() })
    }

    fn choose(&self, _kind: HotOpenKind) -> Work<'_, Option<String>> {
        Box::pin(async { None })
    }
}
