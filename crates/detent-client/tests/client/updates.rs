//! The Software Update flow: Check for Updates asks the service and reads it again.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{UpdateStatus, UpdateWire, UpdatesInput, UpdatesState};
use detent_system::updates::{FakeLeaves, FakeUpdates, UpdatesAsked};
use std::sync::Arc;

fn idle() -> UpdateStatus {
    let wire = UpdateWire {
        state: "idle".to_owned(),
        current: "1.0.0".to_owned(),
        ..UpdateWire::default()
    };
    match UpdateStatus::of_wire(wire) {
        Ok(status) => status,
        Err(absence) => panic!("a plain idle status reads: {absence:?}"),
    }
}

#[tokio::test]
async fn check_for_updates_is_asked_of_the_service() {
    let link = FakeUpdates::answering(idle());
    let client = builder()
        .with_updates(Arc::new(link.clone()))
        .with_log_out(Arc::new(FakeLeaves::default()))
        .build();
    let updates = client.updates();

    updates.send(UpdatesInput::Heard(Ok(idle()))).await;
    updates.send(UpdatesInput::Check).await;

    assert_eq!(link.asked(), [UpdatesAsked::Check]);
    assert!(matches!(updates.live().state(), UpdatesState::Ready(_)));
}

#[tokio::test]
async fn a_refused_check_is_said_in_the_services_words() {
    let link = FakeUpdates::answering(idle());
    link.refuses("Updates are paused.");
    let client = builder().with_updates(Arc::new(link.clone())).build();
    let updates = client.updates();

    updates.send(UpdatesInput::Heard(Ok(idle()))).await;
    updates.send(UpdatesInput::Check).await;

    assert_eq!(link.asked(), [UpdatesAsked::Check]);
}
