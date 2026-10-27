//! Joining from the Wi-Fi page: a network joined at once, a password and an account asked for, a
//! network that hides its name, a refused password, and forgetting a known network.

use super::{FORGET, JOINED, OTHER, PASSWORD, join_button, listed, point, ssid, status, world};
use crate::support::drive::{click, keys, settle, showing, type_into};
use crate::support::wifi::{Asked, Wifi};
use detent_model::{
    HiddenLogin, HiddenNetwork, JoinFailure, NetworkLink, NetworkLock, Passphrase, WifiLogin,
    WifiRadio, WifiStatus, WifiWire,
};

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
    // More keys into the field that has focus: a second click would move the caret.
    keys(&mut harness, "er than that");
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
fn a_network_that_signs_in_asks_for_a_username_and_a_password() {
    let mut wire = WifiWire {
        radio: WifiRadio::On,
        access_points: vec![point(
            "Office",
            90,
            NetworkLock::Secured,
            NetworkLink::Joined,
        )],
        saved: vec![ssid("Office")],
    };
    wire.access_points.push(point(
        "Campus",
        70,
        NetworkLock::Enterprise,
        NetworkLink::Idle,
    ));
    let service = Wifi::answering(WifiStatus::of_wire(wire));
    let (_sandbox, mut harness) = world("wifi-sign-in", &service);
    listed(&mut harness);
    click(&mut harness, &join_button("Campus"));
    showing(&mut harness, "the sign-in sheet", PASSWORD);
    assert_eq!(
        service.asked(),
        [],
        "nothing is asked until the person signs in"
    );
    let join = format!("{PASSWORD} [aria-label=\"Join\"]");
    type_into(&mut harness, "#wifi-username", "ada");
    assert_eq!(
        harness.attr(&join, "aria-disabled").as_deref(),
        Some("true"),
        "the password is missing"
    );
    type_into(&mut harness, "#wifi-password", "analytical");
    click(&mut harness, &join);
    assert_eq!(
        service.asked(),
        [Asked::SignIn(
            ssid("Campus"),
            WifiLogin::new("ada", "analytical").expect("an account")
        )]
    );
    settle(&mut harness, "the sheet closed", |h| h.count(PASSWORD) == 0);
}

#[test]
fn a_known_network_is_forgotten_only_after_the_person_confirms() {
    let service = Wifi::answering(status(WifiRadio::On));
    let (_sandbox, mut harness) = world("wifi-forget", &service);
    listed(&mut harness);
    let forget = "[aria-label=\"Forget Home\"]";
    assert_eq!(harness.count(forget), 1);
    assert_eq!(
        harness.count("[aria-label=\"Forget Neighbour\"]"),
        0,
        "a network that is not known has nothing to forget"
    );
    click(&mut harness, forget);
    showing(&mut harness, "the question", FORGET);
    assert_eq!(
        service.asked(),
        [],
        "nothing is forgotten until the person says so"
    );
    click(&mut harness, &format!("{FORGET} [aria-label=\"Forget\"]"));
    assert_eq!(service.asked(), [Asked::Forget(ssid("Home"))]);
    settle(&mut harness, "the sheet closed", |h| h.count(FORGET) == 0);
    settle(&mut harness, "Home no longer known", |h| {
        h.count(forget) == 0
    });
}

#[test]
fn a_network_that_hides_its_name_is_joined_from_the_other_row() {
    let service = Wifi::answering(status(WifiRadio::On));
    let (_sandbox, mut harness) = world("wifi-hidden", &service);
    listed(&mut harness);
    click(&mut harness, "[aria-label=\"Join another network\"]");
    showing(&mut harness, "the sheet", OTHER);
    let join = format!("{OTHER} [aria-label=\"Join\"]");
    assert_eq!(
        harness.attr(&join, "aria-disabled").as_deref(),
        Some("true"),
        "a network has a name"
    );
    type_into(&mut harness, "#wifi-name", "Back Room");
    click(&mut harness, &join);
    assert_eq!(
        service.asked(),
        [Asked::JoinHidden(HiddenNetwork {
            ssid: ssid("Back Room"),
            login: HiddenLogin::Open,
        })]
    );
    settle(&mut harness, "the sheet closed", |h| h.count(OTHER) == 0);
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
