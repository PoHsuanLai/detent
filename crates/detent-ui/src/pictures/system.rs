//! The pictures of this computer: the system's folders and the added ones, their small pictures in
//! the shared thumbnail cache, and the desktop's own file chooser.

use super::{Pictures, Thumbnails};
use crate::chooser::{Want, choose};
use crate::live::BusSource;
use detent_files::PictureDirs;
use detent_model::{Picture, Work, is_picture};
use std::path::PathBuf;
use std::sync::Arc;

/// The folders on disk, their small pictures, and the chooser on the bus.
#[derive(Debug)]
pub(crate) struct OnDisk {
    bus: BusSource,
    dirs: PictureDirs,
    thumbs: Arc<Thumbnails>,
}

impl OnDisk {
    /// The pictures in `dirs`, made small into `thumbs`, the chooser looked for on `bus`.
    pub(crate) fn new(bus: BusSource, dirs: PictureDirs, thumbs: Thumbnails) -> OnDisk {
        OnDisk {
            bus,
            dirs,
            thumbs: Arc::new(thumbs),
        }
    }
}

impl Pictures for OnDisk {
    fn list(&self) -> Work<'_, Vec<Picture>> {
        Box::pin(async { self.dirs.list() })
    }

    fn add(&self) -> Work<'_, Option<PathBuf>> {
        Box::pin(async {
            let chosen = PathBuf::from(choose(&self.bus, Want::Picture).await?);
            if !is_picture(&chosen) {
                return None;
            }
            self.dirs.add(&chosen).ok()
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
