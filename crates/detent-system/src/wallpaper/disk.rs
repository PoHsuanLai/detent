//! The pictures of this computer: the system's folders and the added ones, their small pictures in
//! the shared thumbnail cache, and the desktop's own file chooser.

use super::{Pictures, Thumbnails};
use crate::chooser::{Asked, ChooserLink, Want};
use detent_files::PictureDirs;
use detent_model::{AddOutcome, Picture, Work, is_picture};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The folders on disk, their small pictures, and the chooser on the bus.
#[derive(Debug)]
pub struct OnDisk {
    chooser: Arc<dyn ChooserLink>,
    dirs: PictureDirs,
    thumbs: Arc<Thumbnails>,
}

impl OnDisk {
    /// The pictures in `dirs`, made small into `thumbs`, added through `chooser`.
    pub fn new(chooser: Arc<dyn ChooserLink>, dirs: PictureDirs, thumbs: Thumbnails) -> OnDisk {
        OnDisk {
            chooser,
            dirs,
            thumbs: Arc::new(thumbs),
        }
    }
}

impl OnDisk {
    /// `chosen` among the added pictures, or the reason it could not be; each refusal is also
    /// written to the journal.
    fn keep(&self, chosen: &Path) -> AddOutcome {
        let name = chosen
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !is_picture(chosen) {
            eprintln!(
                "detent: wallpaper: {} is not a kind of picture the desktop draws",
                chosen.display()
            );
            return AddOutcome::NotAPicture(name);
        }
        match self.dirs.add(chosen) {
            Ok(kept) => AddOutcome::Added(kept),
            Err(fault) => {
                eprintln!(
                    "detent: wallpaper: could not add {}: {fault}",
                    chosen.display()
                );
                AddOutcome::CouldNotCopy(name)
            }
        }
    }
}

impl Pictures for OnDisk {
    fn list(&self) -> Work<'_, Vec<Picture>> {
        Box::pin(async { self.dirs.list() })
    }

    fn add(&self) -> Work<'_, AddOutcome> {
        Box::pin(async {
            match self.chooser.ask(Want::Picture).await {
                Asked::Path(chosen) => self.keep(&PathBuf::from(chosen)),
                Asked::Closed => AddOutcome::Closed,
                Asked::Unavailable => {
                    eprintln!("detent: wallpaper: no file chooser to ask");
                    AddOutcome::NoChooser
                }
            }
        })
    }

    fn thumbnail(&self, source: PathBuf) -> Work<'_, Option<PathBuf>> {
        let thumbs = Arc::clone(&self.thumbs);
        Box::pin(async move {
            // Decoding a wallpaper takes a while: on a thread of its own, never the window's.
            tokio::task::spawn_blocking(move || thumbs.of(&source).ok())
                .await
                .ok()
                .flatten()
        })
    }

    fn roots(&self) -> Vec<PathBuf> {
        let mut roots = self.dirs.folders();
        roots.push(self.thumbs.folder());
        roots
    }
}
