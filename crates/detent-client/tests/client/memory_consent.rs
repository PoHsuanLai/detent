//! The memory-consent flow: a looser value that waits is confirmed when the person presses
//! Confirm, and then waits no more.

use crate::support::builder;
use almanac_core::WaitingSetting;
use detent_model::{Confinement, SettingPair};
use detent_system::memory_consent::FakeMemory;
use std::sync::Arc;

#[tokio::test]
async fn confirming_a_waiting_value_tells_memory_and_clears_it() {
    let key = "memory.retention.search_days";
    let waiting = WaitingSetting {
        key: key.to_owned(),
        waiting: "90".to_owned(),
        shipped: "7".to_owned(),
    };
    let link = FakeMemory::holding(vec![waiting], Confinement::Confined);
    let client = builder().with_memory(Arc::new(link.clone())).build();
    let consent = client.memory_consent();

    consent.refresh().await;
    assert!(consent.state().waiting_for(key).is_some());

    consent.confirm(key).await;

    assert_eq!(
        link.confirmed(),
        [vec![SettingPair {
            key: key.to_owned(),
            value: "90".to_owned()
        }]]
    );
    assert!(consent.state().waiting_for(key).is_none());
}
