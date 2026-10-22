//! This computer's manifests of installed apps, and the desktop's own file chooser.

use super::CornerDesk;
use crate::chooser::{Want, choose};
use crate::live::BusSource;
use detent_files::read_manifests;
use detent_model::{HotActionChoice, HotOpenKind, Work, actions_of_all};
use std::path::PathBuf;

/// The manifest folders on disk and the chooser on the bus.
#[derive(Debug)]
pub(crate) struct OnSystem {
    bus: BusSource,
    intent_dirs: Vec<PathBuf>,
}

impl OnSystem {
    /// The manifests under `intent_dirs` (the folders of trust, first first), the chooser looked
    /// for on `bus`.
    pub(crate) fn new(bus: BusSource, intent_dirs: Vec<PathBuf>) -> OnSystem {
        OnSystem { bus, intent_dirs }
    }
}

impl CornerDesk for OnSystem {
    fn actions(&self) -> Work<'_, Vec<HotActionChoice>> {
        let dirs = self.intent_dirs.clone();
        Box::pin(async move {
            // Reading the files takes a while: on a thread of its own, never the window's.
            tokio::task::spawn_blocking(move || {
                let texts = read_manifests(&dirs);
                actions_of_all(texts.iter().map(String::as_str))
            })
            .await
            .unwrap_or_default()
        })
    }

    fn choose(&self, kind: HotOpenKind) -> Work<'_, Option<String>> {
        Box::pin(async move {
            let want = match kind {
                HotOpenKind::File => Want::File,
                HotOpenKind::Folder => Want::Folder,
                HotOpenKind::App | HotOpenKind::Web => return None,
            };
            choose(&self.bus, want).await
        })
    }
}
