//! The Wi-Fi page over a network service that answers from memory: the switch, the networks, joining
//! one (with a password sheet for a new secured one), the details of the current one, and the page
//! without a service.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, settle, showing, sidebar_row, title_of, type_into};
use crate::support::wifi::{Asked, Wifi};
use detent_model::{
    AccessPoint, JoinFailure, Missing, NetworkLink, NetworkLock, Passphrase, SignalStrength, Ssid,
    WifiRadio, WifiStatus, WifiWire,
};
use detent_ui::Opening;
use ds_harness::Harness;

const PAGE: &str = "[data-system=\"wifi\"]";
const TOGGLE: &str = "[data-system=\"wifi\"] .ds-toggle";
const JOINED: &str = "#wifi-joined";
const PASSWORD: &str = "[data-system=\"password\"]";
const DETAILS: &str = "[data-system=\"details\"]";
const SECURED: &str = "[data-lock=\"secured\"]";
const OPEN: &str = "[data-lock=\"open\"]";

fn wifi() -> Opening {
    Opening {
        page: Some(detent_model::PageSlug("wifi".to_owned())),
        query: String::new(),
    }
}

fn ssid(name: &str) -> Ssid {
    Ssid(name.to_owned())
}

fn point(name: &str, strength: u8, lock: NetworkLock, link: NetworkLink) -> AccessPoint {
    AccessPoint {
        ssid: ssid(name),
        strength: SignalStrength(strength),
        lock,
        link,
    }
}

/// On Office, remembering Home, and seeing an open Cafe and a secured Neighbour.
fn status(radio: WifiRadio) -> WifiStatus {
    WifiStatus::of_wire(WifiWire {
        radio,
        access_points: vec![
            point("Office", 90, NetworkLock::Secured, NetworkLink::Joined),
            point("Home", 70, NetworkLock::Secured, NetworkLink::Idle),
            point("Cafe", 80, NetworkLock::Open, NetworkLink::Idle),
            point("Neighbour", 60, NetworkLock::Secured, NetworkLink::Idle),
        ],
        saved: vec![ssid("Office"), ssid("Home")],
    })
}

fn world(name: &str, service: &Wifi) -> (Sandbox, Harness) {
    let sandbox = Sandbox::new(name).with_wifi(service.clone());
    let harness = sandbox.harness(wifi());
    (sandbox, harness)
}

fn listed(harness: &mut Harness) {
    showing(harness, "the page", PAGE);
    showing(harness, "the joined network", JOINED);
}

fn join_button(name: &str) -> String {
    format!("[aria-label=\"Join {name}\"]")
}

#[test]
fn the_page_is_first_in_the_sidebar_and_says_plainly_when_nothing_serves_wifi() {
    let sandbox = Sandbox::new("wifi-absent");
    let mut harness = sandbox.harness(wifi());
    showing(
        &mut harness,
        "the unavailable row",
        "[data-system=\"unavailable\"]",
    );
    assert_eq!(page_of(&harness).as_deref(), Some("wifi"));
    assert_eq!(title_of(&harness).as_deref(), Some("Wi-Fi"));
    assert_eq!(harness.text_of(&sidebar_row(1)).as_deref(), Some("Wi-Fi"));
    assert_eq!(
        harness.text_of(&sidebar_row(2)).as_deref(),
        Some("Bluetooth")
    );
    assert_eq!(
        harness.text_of(&sidebar_row(5)).as_deref(),
        Some("Displays")
    );
    let row = harness.text_of(".ds-field-row").unwrap_or_default();
    assert!(
        row.contains("Wi-Fi isn\u{2019}t available right now."),
        "{row}"
    );
}

#[test]
fn a_computer_without_wifi_says_so() {
    let service = Wifi::missing(Missing::NoDevice);
    let (_sandbox, mut harness) = world("wifi-no-device", &service);
    showing(
        &mut harness,
        "the unavailable row",
        "[data-system=\"unavailable\"]",
    );
    let row = harness.text_of(".ds-field-row").unwrap_or_default();
    assert!(row.contains("This computer has no Wi-Fi."), "{row}");
}

#[test]
fn the_network_the_computer_is_on_comes_first_with_its_details_and_the_others_follow() {
    let service = Wifi::answering(status(WifiRadio::On));
    let (_sandbox, mut harness) = world("wifi-list", &service);
    listed(&mut harness);
    let joined = harness.text_of(JOINED).unwrap_or_default();
    assert!(
        joined.contains("Office") && joined.contains("Connected"),
        "{joined}"
    );
    assert_eq!(harness.count("[aria-label=\"Details\u{2026}\"]"), 1);
    for name in ["Home", "Cafe", "Neighbour"] {
        assert_eq!(harness.count(&join_button(name)), 1, "{name}");
    }
    assert_eq!(
        harness.count(&join_button("Office")),
        0,
        "the joined one has no Join"
    );
    let titles = harness.html();
    assert!(titles.contains("Known Networks") && titles.contains("Other Networks"));
    assert_eq!(
        harness.count(SECURED),
        3,
        "Office, Home and Neighbour have a lock"
    );
    assert_eq!(harness.count(OPEN), 1, "Cafe has none");
}

#[test]
fn the_switch_asks_the_service_and_the_list_goes_when_the_service_says_it_is_off() {
    let service = Wifi::answering(status(WifiRadio::On));
    let (_sandbox, mut harness) = world("wifi-switch", &service);
    listed(&mut harness);
    assert_eq!(
        harness.attr(TOGGLE, "aria-checked").as_deref(),
        Some("true")
    );
    let groups = harness.count(".ds-field-group");
    click(&mut harness, TOGGLE);
    assert_eq!(service.asked(), [Asked::Radio(WifiRadio::Off)]);
    settle(&mut harness, "the switch off", |h| {
        h.attr(TOGGLE, "aria-checked").as_deref() == Some("false")
    });
    assert_eq!(
        groups, 4,
        "the switch, the joined network, the known and the other networks"
    );
    assert_eq!(
        harness.count(".ds-field-group"),
        1,
        "only the switch is left"
    );
}

#[test]
fn a_switch_that_hardware_holds_off_is_disabled_and_says_why() {
    let service = Wifi::answering(status(WifiRadio::Blocked));
    let (_sandbox, mut harness) = world("wifi-blocked", &service);
    showing(&mut harness, "the page", PAGE);
    assert_eq!(
        harness.attr(TOGGLE, "aria-disabled").as_deref(),
        Some("true")
    );
    assert!(
        harness
            .html()
            .contains("Wi-Fi is switched off on this computer.")
    );
    click(&mut harness, TOGGLE);
    assert_eq!(
        service.asked(),
        [],
        "a switch the hardware holds is not pressed"
    );
}

#[test]
fn an_open_or_remembered_network_is_joined_at_once_with_no_password() {
    let service = Wifi::answering(status(WifiRadio::On));
    let (_sandbox, mut harness) = world("wifi-join-open", &service);
    listed(&mut harness);
    click(&mut harness, &join_button("Cafe"));
    assert_eq!(service.asked(), [Asked::Join(ssid("Cafe"), None)]);
    assert_eq!(harness.count(PASSWORD), 0, "no sheet for an open network");
    settle(&mut harness, "Cafe joined", |h| {
        h.text_of(JOINED).unwrap_or_default().contains("Cafe")
    });

    click(&mut harness, &join_button("Home"));
    assert_eq!(service.asked()[1], Asked::Join(ssid("Home"), None));
    assert_eq!(
        harness.count(PASSWORD),
        0,
        "a remembered network keeps its password"
    );
}

#[test]
fn a_new_secured_network_asks_for_its_password_and_joins_with_it() {
    let service = Wifi::answering(status(WifiRadio::On));
    let (_sandbox, mut harness) = world("wifi-join-secured", &service);
    listed(&mut harness);
    click(&mut harness, &join_button("Neighbour"));
    showing(&mut harness, "the password sheet", PASSWORD);
    assert_eq!(
        service.asked(),
        [],
        "nothing is asked until the password is given"
    );
    let sheet = harness.text_of(PASSWORD).unwrap_or_default();
    assert!(sheet.contains("\u{201c}Neighbour\u{201d}"), "{sheet}");
    let join = format!("{PASSWORD} [aria-label=\"Join\"]");
    assert_eq!(
        harness.attr(&join, "aria-disabled").as_deref(),
        Some("true")
    );

    type_into(&mut harness, &format!("{PASSWORD} .ds-input"), "short");
    assert_eq!(
        harness.attr(&join, "aria-disabled").as_deref(),
        Some("true"),
        "a WPA password has eight characters at least"
    );
    type_into(
        &mut harness,
        &format!("{PASSWORD} .ds-input"),
        "er than that",
    );
    assert_ne!(
        harness.attr(&join, "aria-disabled").as_deref(),
        Some("true")
    );
    click(&mut harness, &join);
    assert_eq!(
        service.asked(),
        [Asked::Join(
            ssid("Neighbour"),
            Some(Passphrase::new("shorter than that"))
        )]
    );
    settle(&mut harness, "the sheet closed", |h| h.count(PASSWORD) == 0);
    settle(&mut harness, "Neighbour joined", |h| {
        h.text_of(JOINED).unwrap_or_default().contains("Neighbour")
    });
}

#[test]
fn a_password_the_network_refuses_asks_again_and_says_so() {
    let service = Wifi::answering(status(WifiRadio::On));
    service.join_fails(JoinFailure::WrongPassword);
    let (_sandbox, mut harness) = world("wifi-wrong-password", &service);
    listed(&mut harness);
    click(&mut harness, &join_button("Neighbour"));
    showing(&mut harness, "the password sheet", PASSWORD);
    type_into(
        &mut harness,
        &format!("{PASSWORD} .ds-input"),
        "not the one",
    );
    click(&mut harness, &format!("{PASSWORD} [aria-label=\"Join\"]"));
    assert_eq!(service.asked().len(), 1);
    settle(&mut harness, "the question again", |h| {
        h.text_of(PASSWORD)
            .unwrap_or_default()
            .contains("The password is incorrect. Try again.")
    });
    assert!(
        harness
            .text_of(JOINED)
            .unwrap_or_default()
            .contains("Office"),
        "the computer is still on the network it was on"
    );
}

#[test]
fn any_other_failure_to_join_is_said_in_a_sentence() {
    let service = Wifi::answering(status(WifiRadio::On));
    service.join_fails(JoinFailure::Told(
        "Couldn\u{2019}t join \u{201c}Cafe\u{201d}.".to_owned(),
    ));
    let (_sandbox, mut harness) = world("wifi-join-fails", &service);
    listed(&mut harness);
    click(&mut harness, &join_button("Cafe"));
    settle(&mut harness, "the sentence", |h| {
        h.html()
            .contains("Couldn\u{2019}t join \u{201c}Cafe\u{201d}.")
    });
    assert_eq!(harness.count(PASSWORD), 0);
}

#[test]
fn the_details_of_the_current_network_open_in_a_sheet_and_done_closes_it() {
    let service = Wifi::answering(status(WifiRadio::On));
    let (_sandbox, mut harness) = world("wifi-details", &service);
    listed(&mut harness);
    click(&mut harness, "[aria-label=\"Details\u{2026}\"]");
    showing(&mut harness, "the details", DETAILS);
    let sheet = harness.text_of(DETAILS).unwrap_or_default();
    for want in ["Office", "Password protected", "Good (90 percent)"] {
        assert!(sheet.contains(want), "{want} in {sheet}");
    }
    click(&mut harness, &format!("{DETAILS} [aria-label=\"Done\"]"));
    settle(&mut harness, "the sheet closed", |h| h.count(DETAILS) == 0);
}
