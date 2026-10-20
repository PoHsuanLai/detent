//! The pictures of this computer: the system's folders and the added ones, and the desktop's own
//! file chooser.

use super::Pictures;
use crate::chooser::{Want, choose};
use crate::live::BusSource;
use detent_files::PictureDirs;
use detent_model::{Work, is_picture};
use std::path::PathBuf;

/// The folders on disk and the chooser on the bus.
#[derive(Debug)]
pub(crate) struct OnDisk {
    bus: BusSource,
    dirs: PictureDirs,
}

impl OnDisk {
    /// The pictures in `dirs`, the chooser looked for on `bus`.
    pub(crate) fn new(bus: BusSource, dirs: PictureDirs) -> OnDisk {
        OnDisk { bus, dirs }
    }
}

impl Pictures for OnDisk {
    fn list(&self) -> Work<'_, Vec<PathBuf>> {
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
}
