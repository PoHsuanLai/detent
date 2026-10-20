//! What the picture grid asks for.

use detent_model::Work;
use std::fmt::Debug;
use std::path::PathBuf;

/// The things the grid asks for.
pub trait Pictures: Debug + Send + Sync {
    /// Every picture on offer: the system's, then the ones the person added.
    fn list(&self) -> Work<'_, Vec<PathBuf>>;
    /// Asks the person for a picture and keeps it among the added ones; the path to store, or
    /// `None` when the chooser was closed, there is none, or what was chosen is not a picture.
    fn add(&self) -> Work<'_, Option<PathBuf>>;
}

/// No pictures and no chooser.
#[derive(Debug, Clone, Copy)]
pub struct NoPictures;

impl Pictures for NoPictures {
    fn list(&self) -> Work<'_, Vec<PathBuf>> {
        Box::pin(async { Vec::new() })
    }

    fn add(&self) -> Work<'_, Option<PathBuf>> {
        Box::pin(async { None })
    }
}
