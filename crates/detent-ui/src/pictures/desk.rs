//! What the picture grid asks for.

use detent_model::{Picture, Work};
use std::fmt::Debug;
use std::path::PathBuf;

/// The things the grid asks for.
pub trait Pictures: Debug + Send + Sync {
    /// Every picture on offer: the system's, then the ones the person added.
    fn list(&self) -> Work<'_, Vec<Picture>>;
    /// Asks the person for a picture and keeps it among the added ones; the path to store, or
    /// `None` when the chooser was closed, there is none, or what was chosen is not a picture.
    fn add(&self) -> Work<'_, Option<PathBuf>>;
    /// The small picture of the file at `source` for a tile, made now when there is none yet: the
    /// path of a file the window may draw, or `None` when the file cannot be made small.
    fn thumbnail(&self, source: PathBuf) -> Work<'_, Option<PathBuf>>;
    /// The folders the window may draw pictures from: where the pictures are and where their
    /// small pictures are kept. A launcher names them to the window; a fake names none.
    fn roots(&self) -> Vec<PathBuf>;
}

/// No pictures and no chooser.
#[derive(Debug, Clone, Copy)]
pub struct NoPictures;

impl Pictures for NoPictures {
    fn list(&self) -> Work<'_, Vec<Picture>> {
        Box::pin(async { Vec::new() })
    }

    fn add(&self) -> Work<'_, Option<PathBuf>> {
        Box::pin(async { None })
    }

    fn thumbnail(&self, _source: PathBuf) -> Work<'_, Option<PathBuf>> {
        Box::pin(async { None })
    }

    fn roots(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}
