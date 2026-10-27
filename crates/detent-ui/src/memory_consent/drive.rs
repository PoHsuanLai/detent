//! The looser memory settings the window follows: what waits, what a save confirms, and what the
//! Confirm button sends. What counts as looser is memory's to say; this only asks it.

use crate::env::Env;
use crate::files::Edit;
use detent_model::{Confinement, ConsentState, SettingPair, is_memory_setting, to_confirm};
use detent_system::memory_consent::{MemoryFailure, MemoryLink};
use dioxus::prelude::*;
use std::sync::Arc;

/// The state of the looser memory settings and the link to reach memory through.
#[derive(Clone)]
pub(crate) struct MemoryConsent {
    memory: Arc<dyn MemoryLink>,
    state: Signal<ConsentState>,
}

/// The window's hold on the looser memory settings.
pub(crate) fn use_memory_consent() -> MemoryConsent {
    let env = use_context::<Env>();
    MemoryConsent {
        memory: env.memory.clone(),
        state: use_signal(ConsentState::default),
    }
}

impl MemoryConsent {
    /// What waits and what failed, as of the last read.
    pub(crate) fn state(&self) -> ConsentState {
        (self.state)()
    }

    fn update(&self, change: impl FnOnce(ConsentState) -> ConsentState) {
        let mut state = self.state;
        let next = change(state.peek().clone());
        state.set(next);
    }

    /// Reads again what waits. Memory not running, or not answering, leaves nothing waiting.
    pub(crate) async fn refresh(&self) {
        let found = self.memory.waiting().await.unwrap_or_default();
        self.update(|state| state.listed(found));
    }

    /// Reads again how far memory confines itself. Memory not running, not answering, or not
    /// having the property leaves nothing to say.
    pub(crate) async fn look_at_confinement(&self) {
        let found = self
            .memory
            .confinement()
            .await
            .unwrap_or(Confinement::Unknown);
        self.update(|state| state.confined(found));
    }

    /// The person saved `edit`. When it is a memory setting and memory reports that value as
    /// looser than the shipped one, saving it is the person's word, and memory is told.
    pub(crate) fn saved(&self, edit: &Edit) {
        if !is_memory_setting(&edit.entry) {
            return;
        }
        let set = SettingPair::of(&edit.entry, &edit.value);
        let me = self.clone();
        spawn(async move { me.settle(set).await });
    }

    /// The person pressed Confirm on the value of `key` that waits.
    pub(crate) fn confirm(&self, key: &str) {
        let Some(held) = self.state().waiting_for(key).cloned() else {
            return;
        };
        let pair = SettingPair {
            key: held.key,
            value: held.waiting,
        };
        let me = self.clone();
        spawn(async move { me.send(vec![pair]).await });
    }

    async fn settle(&self, set: SettingPair) {
        let found = self.memory.waiting().await.unwrap_or_default();
        let sure = to_confirm(&found, &[set]);
        self.update(|state| state.listed(found));
        if !sure.is_empty() {
            self.send(sure).await;
        }
    }

    async fn send(&self, pairs: Vec<SettingPair>) {
        let keys: Vec<String> = pairs.iter().map(|pair| pair.key.clone()).collect();
        match self.memory.confirm(pairs).await {
            Ok(()) => self.update(|state| state.cleared(&keys)),
            Err(MemoryFailure::Trouble(trouble)) => {
                self.update(|state| state.troubled(&keys, trouble))
            }
            Err(MemoryFailure::Absent) => {}
        }
        self.refresh().await;
    }
}
