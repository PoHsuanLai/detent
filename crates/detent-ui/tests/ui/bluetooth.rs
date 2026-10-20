//! The Bluetooth page over a BlueZ that answers from memory: the switch, My Devices and Nearby
//! Devices, connecting, pairing with a number to compare, forgetting after a question, and looking
//! for devices only while the page is open.

use crate::support::Sandbox;
use crate::support::bluetooth::{Asked, Bluetooth};
use crate::support::drive::{click, page_of, settle, showing, sidebar_row, title_of, type_into};
use detent_model::{
    BatteryLevel, BtAddress, BtDevice, BtKind, BtLink, BtPairing, BtPower, BtStatus, Missing,
    PairingAnswer, PairingAsk, PairingCode, PairingQuestion,
};
use detent_ui::Opening;
use ds_harness::Harness;

const PAGE: &str = "[data-system=\"bluetooth\"]";
const TOGGLE: &str = "[data-system=\"bluetooth\"] .ds-toggle";
const FORGET: &str = "[data-system=\"forget\"]";
const PAIRING: &str = "[data-system=\"pairing\"]";

fn bluetooth() -> Opening {
    Opening {
        page: Some(detent_model::PageSlug("bluetooth".to_owned())),
        query: String::new(),
    }
}

fn address(last: u8) -> BtAddress {
    BtAddress(format!("AA:BB:CC:DD:EE:{last:02X}"))
}

fn device(last: u8, name: &str, kind: BtKind, link: BtLink, pairing: BtPairing) -> BtDevice {
    BtDevice {
        address: address(last),
        name: name.to_owned(),
        kind,
        link,
        pairing,
        battery: None,
    }
}

/// Paired with a connected mouse and idle headphones, and a nearby game pad it has not paired with.
fn status(power: BtPower) -> BtStatus {
    BtStatus::new(
        power,
        vec![
            device(
                1,
                "Quiet Mouse",
                BtKind::Mouse,
                BtLink::Connected,
                BtPairing::Paired,
            ),
            device(
                2,
                "Big Headphones",
                BtKind::Audio,
                BtLink::Disconnected,
                BtPairing::Paired,
            ),
            device(
                3,
                "Nearby Pad",
                BtKind::Gamepad,
                BtLink::Disconnected,
                BtPairing::Unpaired,
            ),
        ],
    )
}

fn world(name: &str, service: &Bluetooth) -> (Sandbox, Harness) {
    let sandbox = Sandbox::new(name).with_bluetooth(service.clone());
    let harness = sandbox.harness(bluetooth());
    (sandbox, harness)
}

fn button(verb: &str, name: &str) -> String {
    format!("[aria-label=\"{verb} {name}\"]")
}

#[test]
fn the_page_is_second_in_the_sidebar_and_says_plainly_when_nothing_serves_bluetooth() {
    let sandbox = Sandbox::new("bluetooth-absent");
    let mut harness = sandbox.harness(bluetooth());
    showing(
        &mut harness,
        "the unavailable row",
        "[data-system=\"unavailable\"]",
    );
    assert_eq!(page_of(&harness).as_deref(), Some("bluetooth"));
    assert_eq!(title_of(&harness).as_deref(), Some("Bluetooth"));
    assert_eq!(
        harness.text_of(&sidebar_row(2)).as_deref(),
        Some("Bluetooth")
    );
    let row = harness.text_of(".ds-field-row").unwrap_or_default();
    assert!(
        row.contains("Bluetooth isn\u{2019}t available right now."),
        "{row}"
    );
}

#[test]
fn a_computer_without_bluetooth_says_so() {
    let service = Bluetooth::missing(Missing::NoDevice);
    let (_sandbox, mut harness) = world("bluetooth-no-device", &service);
    showing(
        &mut harness,
        "the unavailable row",
        "[data-system=\"unavailable\"]",
    );
    let row = harness.text_of(".ds-field-row").unwrap_or_default();
    assert!(row.contains("This computer has no Bluetooth."), "{row}");
}

#[test]
fn devices_are_listed_as_mine_and_nearby_with_whether_they_are_connected() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-list", &service);
    showing(&mut harness, "the page", PAGE);
    showing(
        &mut harness,
        "the devices",
        "[aria-label=\"Connect Nearby Pad\"]",
    );
    let html = harness.html();
    assert!(html.contains("My Devices") && html.contains("Nearby Devices"));
    let mine = harness.text_of(PAGE).unwrap_or_default();
    assert!(
        mine.contains("Connected") && mine.contains("Not Connected"),
        "{mine}"
    );
    assert_eq!(harness.count(&button("Disconnect", "Quiet Mouse")), 1);
    assert_eq!(harness.count(&button("Connect", "Big Headphones")), 1);
    assert_eq!(harness.count(&button("Forget", "Quiet Mouse")), 1);
    assert_eq!(
        harness.count(&button("Forget", "Nearby Pad")),
        0,
        "only a paired device is forgotten"
    );
}

#[test]
fn the_switch_asks_bluez_and_the_lists_go_when_it_says_it_is_off() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-switch", &service);
    showing(&mut harness, "the page", PAGE);
    assert_eq!(
        harness.attr(TOGGLE, "aria-checked").as_deref(),
        Some("true")
    );
    click(&mut harness, TOGGLE);
    assert_eq!(service.asked(), [Asked::Power(BtPower::Off)]);
    settle(&mut harness, "the switch off", |h| {
        h.attr(TOGGLE, "aria-checked").as_deref() == Some("false")
    });
    assert_eq!(
        harness.count(".ds-field-group"),
        1,
        "only the switch is left"
    );
}

#[test]
fn connecting_and_disconnecting_ask_bluez_and_the_row_follows() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-connect", &service);
    showing(
        &mut harness,
        "the devices",
        "[aria-label=\"Connect Big Headphones\"]",
    );
    click(&mut harness, &button("Connect", "Big Headphones"));
    assert_eq!(service.asked(), [Asked::Connect(address(2))]);
    showing(
        &mut harness,
        "headphones connected",
        "[aria-label=\"Disconnect Big Headphones\"]",
    );

    click(&mut harness, &button("Disconnect", "Quiet Mouse"));
    assert_eq!(service.asked()[1], Asked::Disconnect(address(1)));
    showing(
        &mut harness,
        "mouse disconnected",
        "[aria-label=\"Connect Quiet Mouse\"]",
    );
}

#[test]
fn a_nearby_device_pairs_after_the_person_confirms_the_number() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-pair", &service);
    showing(
        &mut harness,
        "the nearby device",
        "[aria-label=\"Connect Nearby Pad\"]",
    );
    settle(&mut harness, "a session", |_| service.looking() == 1);

    let mut answered = service.ask(PairingQuestion {
        address: address(3),
        name: "Nearby Pad".to_owned(),
        ask: PairingAsk::Compare(PairingCode(4_217)),
    });
    showing(&mut harness, "the question", PAIRING);
    let sheet = harness.text_of(PAIRING).unwrap_or_default();
    assert!(
        sheet.contains("004217") && sheet.contains("\u{201c}Nearby Pad\u{201d}"),
        "{sheet}"
    );
    assert!(
        answered.try_recv().is_err(),
        "nothing is answered until the person does"
    );
    click(&mut harness, &format!("{PAIRING} [aria-label=\"Connect\"]"));
    assert_eq!(answered.try_recv().ok(), Some(PairingAnswer::Yes));
    settle(&mut harness, "the sheet closed", |h| h.count(PAIRING) == 0);
}

#[test]
fn a_device_that_tells_its_battery_shows_it_in_its_row() {
    let mut standing = status(BtPower::On);
    for device in &mut standing.devices {
        if device.name == "Quiet Mouse" {
            device.battery = Some(BatteryLevel(80));
        }
    }
    let service = Bluetooth::answering(standing);
    let (_sandbox, mut harness) = world("bluetooth-battery", &service);
    showing(&mut harness, "the battery", "[data-system=\"battery\"]");
    assert_eq!(harness.count("[data-system=\"battery\"]"), 1);
    assert_eq!(
        harness.text_of("[data-system=\"battery\"]").as_deref(),
        Some("80%")
    );
}

#[test]
fn a_pin_is_typed_into_the_sheet_and_sent_to_the_device() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-pin", &service);
    showing(
        &mut harness,
        "the nearby device",
        "[aria-label=\"Connect Nearby Pad\"]",
    );
    settle(&mut harness, "a session", |_| service.looking() == 1);
    let mut answered = service.ask(PairingQuestion {
        address: address(3),
        name: "Nearby Pad".to_owned(),
        ask: PairingAsk::PinCode,
    });
    showing(&mut harness, "the question", PAIRING);
    assert!(
        answered.try_recv().is_err(),
        "nothing is answered until the person types and presses Connect"
    );
    type_into(&mut harness, &format!("{PAIRING} .ds-input"), "0000");
    click(&mut harness, &format!("{PAIRING} [aria-label=\"Connect\"]"));
    assert_eq!(
        answered.try_recv().ok(),
        Some(PairingAnswer::Typed("0000".to_owned()))
    );
}

#[test]
fn cancelling_a_pairing_question_says_no_to_the_device() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-pair-cancel", &service);
    showing(
        &mut harness,
        "the nearby device",
        "[aria-label=\"Connect Nearby Pad\"]",
    );
    settle(&mut harness, "a session", |_| service.looking() == 1);
    let mut answered = service.ask(PairingQuestion {
        address: address(3),
        name: "Nearby Pad".to_owned(),
        ask: PairingAsk::Allow,
    });
    showing(&mut harness, "the question", PAIRING);
    click(&mut harness, &format!("{PAIRING} [aria-label=\"Cancel\"]"));
    assert_eq!(answered.try_recv().ok(), Some(PairingAnswer::No));
}

#[test]
fn pressing_connect_on_a_nearby_device_pairs_with_it_and_it_moves_to_my_devices() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-pair-press", &service);
    showing(
        &mut harness,
        "the nearby device",
        "[aria-label=\"Connect Nearby Pad\"]",
    );
    click(&mut harness, &button("Connect", "Nearby Pad"));
    assert_eq!(service.asked(), [Asked::Pair(address(3))]);
    showing(
        &mut harness,
        "the device kept",
        "[aria-label=\"Forget Nearby Pad\"]",
    );
}

#[test]
fn forgetting_asks_first_and_only_the_confirmation_forgets() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-forget", &service);
    showing(
        &mut harness,
        "the devices",
        "[aria-label=\"Forget Quiet Mouse\"]",
    );
    click(&mut harness, &button("Forget", "Quiet Mouse"));
    showing(&mut harness, "the question", FORGET);
    assert_eq!(service.asked(), [], "the question comes first");
    let sheet = harness.text_of(FORGET).unwrap_or_default();
    assert!(
        sheet.contains("Forget \u{201c}Quiet Mouse\u{201d}?"),
        "{sheet}"
    );

    click(&mut harness, &format!("{FORGET} [aria-label=\"Cancel\"]"));
    settle(&mut harness, "the sheet closed", |h| h.count(FORGET) == 0);
    assert_eq!(service.asked(), [], "Cancel forgets nothing");

    click(&mut harness, &button("Forget", "Quiet Mouse"));
    showing(&mut harness, "the question", FORGET);
    click(
        &mut harness,
        &format!("{FORGET} [aria-label=\"Forget Device\"]"),
    );
    assert_eq!(service.asked(), [Asked::Forget(address(1))]);
    settle(&mut harness, "the device gone", |h| {
        h.count("[aria-label=\"Forget Quiet Mouse\"]") == 0
    });
}

#[test]
fn devices_are_looked_for_only_while_the_page_is_open() {
    let service = Bluetooth::answering(status(BtPower::On));
    let (_sandbox, mut harness) = world("bluetooth-discovery", &service);
    showing(&mut harness, "the page", PAGE);
    settle(&mut harness, "looking", |_| service.looking() == 1);

    click(&mut harness, &sidebar_row(3));
    settle(&mut harness, "another page", |h| {
        page_of(h).as_deref() == Some("appearance")
    });
    settle(&mut harness, "not looking", |_| service.looking() == 0);

    click(&mut harness, &sidebar_row(2));
    settle(&mut harness, "looking again", |_| service.looking() == 1);
}
