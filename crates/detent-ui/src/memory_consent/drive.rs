//! The looser memory settings the window follows: what waits, what a save confirms, and what the
//! Confirm button sends. What counts as looser is memory's to say; the asking is
//! `detent-client`'s, and this holds its state as a signal.

use crate::files::Edit;
use crate::live::{use_client, use_events};
use detent_client::{MemoryConsent as Consent, Pane};
use detent_model::ConsentState;
use dioxus::prelude::*;

/// The window's hold on the looser memory settings.
#[derive(Clone)]
pub(crate) struct MemoryConsent {
    pane: Consent,
    state: Signal<ConsentState>,
}

/// The window's hold on the looser memory settings.
pub(crate) fn use_memory_consent() -> MemoryConsent {
    let client = use_client();
    let pane = use_hook(|| client.memory_consent());
    let state = use_events(pane.live());
    MemoryConsent { pane, state }
}

impl MemoryConsent {
    /// What waits and what failed, as of the last read.
    pub(crate) fn state(&self) -> ConsentState {
        (self.state)()
    }

    /// Reads again what waits. Memory not running, or not answering, leaves nothing waiting.
    pub(crate) async fn refresh(&self) {
        self.pane.refresh().await;
    }

    /// Reads again how far memory confines itself.
    pub(crate) async fn look_at_confinement(&self) {
        self.pane.look_at_confinement().await;
    }

    /// The person saved `edit`. When it is a memory setting and memory reports that value as
    /// looser than the shipped one, saving it is the person's word, and memory is told.
    pub(crate) fn saved(&self, edit: &Edit) {
        spawn(self.pane.saved(&edit.entry, &edit.value));
    }

    /// The person pressed Confirm on the value of `key` that waits.
    pub(crate) fn confirm(&self, key: &str) {
        spawn(self.pane.confirm(key));
    }
}
