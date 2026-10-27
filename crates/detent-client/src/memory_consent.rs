//! The looser memory settings the window follows: what waits, what a save confirms, and what the
//! Confirm button sends. What counts as looser is memory's to say; this only asks it.

use crate::live::Live;
use crate::pane::{Pane, Pending};
use detent_model::{
    Confinement, ConsentState, KeyEntry, SettingPair, is_memory_setting, to_confirm,
};
use detent_system::memory_consent::{MemoryFailure, MemoryLink};
use std::sync::Arc;

/// The state of the looser memory settings and the link to reach memory through.
#[derive(Debug, Clone)]
pub struct MemoryConsent {
    memory: Arc<dyn MemoryLink>,
    live: Live<ConsentState>,
}

impl MemoryConsent {
    pub(crate) fn over(memory: Arc<dyn MemoryLink>) -> MemoryConsent {
        MemoryConsent {
            memory,
            live: Live::default(),
        }
    }

    /// What waits and what failed, as of the last read.
    pub fn state(&self) -> ConsentState {
        self.live.state()
    }

    /// Reads again what waits. Memory not running, or not answering, leaves nothing waiting.
    pub async fn refresh(&self) {
        let found = self.memory.waiting().await.unwrap_or_default();
        self.live.update(|state| state.listed(found));
    }

    /// Reads again how far memory confines itself. Memory not running, not answering, or not
    /// having the property leaves nothing to say.
    pub async fn look_at_confinement(&self) {
        let found = self
            .memory
            .confinement()
            .await
            .unwrap_or(Confinement::Unknown);
        self.live.update(|state| state.confined(found));
    }

    /// The person saved `value` of `entry`. When it is a memory setting and memory reports that
    /// value as looser than the shipped one, saving it is the person's word, and memory is told.
    pub fn saved(&self, entry: &KeyEntry, value: &toml::Value) -> Pending {
        if !is_memory_setting(entry) {
            return Box::pin(std::future::ready(()));
        }
        let set = SettingPair::of(entry, value);
        let me = self.clone();
        Box::pin(async move { me.settle(set).await })
    }

    /// The person pressed Confirm on the value of `key` that waits.
    pub fn confirm(&self, key: &str) -> Pending {
        let Some(held) = self.state().waiting_for(key).cloned() else {
            return Box::pin(std::future::ready(()));
        };
        let pair = SettingPair {
            key: held.key,
            value: held.waiting,
        };
        let me = self.clone();
        Box::pin(async move { me.send(vec![pair]).await })
    }

    async fn settle(&self, set: SettingPair) {
        let found = self.memory.waiting().await.unwrap_or_default();
        let sure = to_confirm(&found, &[set]);
        self.live.update(|state| state.listed(found));
        if !sure.is_empty() {
            self.send(sure).await;
        }
    }

    async fn send(&self, pairs: Vec<SettingPair>) {
        let keys: Vec<String> = pairs.iter().map(|pair| pair.key.clone()).collect();
        match self.memory.confirm(pairs).await {
            Ok(()) => self.live.update(|state| state.cleared(&keys)),
            Err(MemoryFailure::Trouble(trouble)) => {
                self.live.update(|state| state.troubled(&keys, trouble));
            }
            Err(MemoryFailure::Absent) => {}
        }
        self.refresh().await;
    }
}

impl Pane for MemoryConsent {
    type State = ConsentState;

    fn live(&self) -> &Live<ConsentState> {
        &self.live
    }

    /// Reads what waits and how far memory confines itself, once.
    fn watch(&self) -> Pending {
        let me = self.clone();
        Box::pin(async move {
            me.refresh().await;
            me.look_at_confinement().await;
        })
    }
}
