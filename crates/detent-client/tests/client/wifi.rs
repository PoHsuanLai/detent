//! The Wi-Fi flow: a secured network asks for its password, then is joined with it.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{
    AccessPoint, NetworkLink, NetworkLock, Passphrase, SignalStrength, Ssid, WifiAsking, WifiInput,
    WifiRadio, WifiState, WifiStatus, WifiWire,
};
use detent_system::wifi::{FakeWifi, WifiAsked};
use std::sync::Arc;

fn status() -> WifiStatus {
    WifiStatus::of_wire(WifiWire {
        radio: WifiRadio::On,
        access_points: vec![AccessPoint {
            ssid: Ssid("Home".to_owned()),
            strength: SignalStrength(80),
            lock: NetworkLock::Secured,
            link: NetworkLink::Idle,
        }],
        saved: Vec::new(),
    })
}

#[tokio::test]
async fn a_secured_network_asks_for_a_password_and_is_joined_with_it() {
    let link = FakeWifi::answering(status());
    let client = builder().with_wifi(Arc::new(link.clone())).build();
    let wifi = client.wifi();
    let home = Ssid("Home".to_owned());

    wifi.send(WifiInput::Heard(Ok(status()))).await;
    wifi.send(WifiInput::Pick(home.clone())).await;
    let WifiState::Ready(ready) = wifi.live().state() else {
        panic!("the page is ready");
    };
    assert!(matches!(ready.asking, WifiAsking::Password(_)));
    assert!(
        link.asked().is_empty(),
        "nothing is asked of the service yet"
    );

    let password = Passphrase::new("correct horse battery");
    wifi.send(WifiInput::Password(home.clone(), password.clone()))
        .await;
    assert_eq!(
        link.asked(),
        [WifiAsked::Join(home.clone(), Some(password))]
    );
    let WifiState::Ready(ready) = wifi.live().state() else {
        panic!("the page is still ready");
    };
    let joined = ready
        .status
        .networks
        .iter()
        .any(|network| network.ssid == home && network.link == NetworkLink::Joined);
    assert!(joined, "the page shows the network joined");
}
