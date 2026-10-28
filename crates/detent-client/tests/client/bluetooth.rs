//! The Bluetooth flow: a device that was only seen is paired when the person presses Connect.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{BtAddress, BtDevice, BtInput, BtKind, BtLink, BtState, BtStatus};
use detent_system::bluetooth::{BtAsked, FakeBluetooth};
use std::sync::Arc;

fn nearby(address: &BtAddress) -> BtDevice {
    BtDevice {
        address: address.clone(),
        name: "Keyboard".to_owned(),
        kind: BtKind::Keyboard,
        link: BtLink::Disconnected,
        paired: false,
        battery: None,
    }
}

#[tokio::test]
async fn pressing_connect_on_a_nearby_device_pairs_it() {
    let address = BtAddress("AA:BB:CC:DD:EE:FF".to_owned());
    let status = BtStatus::new(true, vec![nearby(&address)]);
    let link = FakeBluetooth::answering(status.clone());
    let client = builder().with_bluetooth(Arc::new(link.clone())).build();
    let bluetooth = client.bluetooth();

    bluetooth.send(BtInput::Heard(Ok(status))).await;
    bluetooth.send(BtInput::Pair(address.clone())).await;

    assert_eq!(link.asked(), [BtAsked::Pair(address)]);
    assert!(matches!(bluetooth.live().state(), BtState::Ready(_)));
}

#[tokio::test]
async fn a_service_that_is_not_there_leaves_the_page_unavailable() {
    let link = FakeBluetooth::missing(detent_model::Absence::NotRunning);
    let client = builder().with_bluetooth(Arc::new(link)).build();
    let bluetooth = client.bluetooth();
    bluetooth
        .send(BtInput::Heard(Err(detent_model::Absence::NotRunning)))
        .await;
    assert!(matches!(bluetooth.live().state(), BtState::Unavailable(_)));
}
