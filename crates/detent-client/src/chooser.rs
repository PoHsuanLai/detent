//! A folder or a picture chosen by the person in the desktop's own chooser, never typed.

use detent_system::chooser::{ChooserLink, choose};
use std::sync::Arc;

pub use detent_system::chooser::{Asked, Want};

/// The desktop's chooser.
#[derive(Debug, Clone)]
pub struct Chooser(Arc<dyn ChooserLink>);

impl Chooser {
    pub(crate) fn over(link: Arc<dyn ChooserLink>) -> Chooser {
        Chooser(link)
    }

    /// Asks the person for a folder, a file or a picture, and waits for the answer; `None` when
    /// the chooser is closed, there is none, or what was chosen is not a file on this computer.
    pub async fn choose(&self, want: Want) -> Option<String> {
        choose(self.0.as_ref(), want).await
    }
}
