//! The Wi-Fi fake moves its standing the way the real service does, and writes down what it was
//! asked.

use detent_model::{
    AccessPoint, NetworkLink, SignalStrength, Ssid, WifiRadio, WifiSecurity, WifiStatus, WifiWire,
};
use detent_system::wifi::{FakeWifi, WifiAsked, WifiLink};

fn status() -> WifiStatus {
    let point = |name: &str| AccessPoint {
        ssid: Ssid(name.to_owned()),
        strength: SignalStrength(80),
        security: WifiSecurity::Open,
        link: NetworkLink::Idle,
    };
    WifiStatus::of_wire(WifiWire {
        radio: WifiRadio::On,
        access_points: vec![point("Home"), point("Cafe")],
        saved: Vec::new(),
    })
}

#[tokio::test]
async fn joining_a_network_joins_it_alone_and_is_written_down() {
    let wifi = FakeWifi::answering(status());
    let home = Ssid("Home".to_owned());
    assert_eq!(wifi.join(home.clone(), None).await, Ok(()));
    let Ok(now) = wifi.read().await else {
        panic!("the fake answers");
    };
    let links: Vec<(String, NetworkLink)> = now
        .networks
        .iter()
        .map(|network| (network.ssid.0.clone(), network.link))
        .collect();
    assert!(links.contains(&("Home".to_owned(), NetworkLink::Joined)));
    assert!(links.contains(&("Cafe".to_owned(), NetworkLink::Idle)));
    assert_eq!(wifi.asked(), [WifiAsked::Join(home, None)]);
}
