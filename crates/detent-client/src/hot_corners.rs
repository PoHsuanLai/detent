//! What the Hot Corners sheet asks of the computer: the actions installed apps offer, and a file
//! or a folder chosen by the person.

use detent_model::{HotActionChoice, HotOpenKind};
use detent_system::hot_corners::CornerDesk;
use std::sync::Arc;

/// The sheet's desk.
#[derive(Debug, Clone)]
pub struct Corners(Arc<dyn CornerDesk>);

impl Corners {
    pub(crate) fn over(desk: Arc<dyn CornerDesk>) -> Corners {
        Corners(desk)
    }

    /// Every action of the installed apps that a corner can run as it is.
    pub async fn actions(&self) -> Vec<HotActionChoice> {
        self.0.actions().await
    }

    /// Asks the person for a file or a folder (`kind` says which; any other kind asks nothing)
    /// and waits: the path chosen, or `None` when they closed the chooser or there is none.
    pub async fn choose(&self, kind: HotOpenKind) -> Option<String> {
        self.0.choose(kind).await
    }
}
