//! The pictures behind the Wallpaper page: the ones on offer, adding one, and the small picture
//! of each for a tile.

use detent_model::{AddOutcome, Picture};
use detent_system::wallpaper::Pictures;
use std::path::PathBuf;
use std::sync::Arc;

/// The pictures' handle: cheap to clone.
#[derive(Debug, Clone)]
pub struct Wallpaper(Arc<dyn Pictures>);

impl Wallpaper {
    pub(crate) fn over(pictures: Arc<dyn Pictures>) -> Wallpaper {
        Wallpaper(pictures)
    }

    /// Every picture on offer: the system's, then the ones the person added, each marked with
    /// which it is.
    pub async fn list(&self) -> Vec<Picture> {
        self.0.list().await
    }

    /// Asks the person for a picture and keeps it among the added ones; how that ended.
    pub async fn add(&self) -> AddOutcome {
        self.0.add().await
    }

    /// The small picture of the file at `source` for a tile, made now when there is none yet: the
    /// path of a file the window may draw, or `None` when the file cannot be made small.
    pub async fn thumbnail(&self, source: PathBuf) -> Option<PathBuf> {
        self.0.thumbnail(source).await
    }

    /// The folders the window may draw pictures from.
    pub fn roots(&self) -> Vec<PathBuf> {
        self.0.roots()
    }
}
