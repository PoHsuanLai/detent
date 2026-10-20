use super::*;
use crate::missing::Missing;

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

fn status(points: Vec<AccessPoint>, saved: &[&str]) -> WifiStatus {
    WifiStatus::of_wire(WifiWire {
        radio: WifiRadio::On,
        access_points: points,
        saved: saved.iter().map(|name| ssid(name)).collect(),
    })
}

fn ready_with(status: WifiStatus) -> WifiState {
    WifiState::Ready(WifiReady {
        status,
        sheet: WifiSheet::Closed,
        joining: None,
    })
}

fn names(status: &WifiStatus) -> Vec<&str> {
    status.networks.iter().map(|n| n.ssid.0.as_str()).collect()
}

fn page_of(state: &WifiState) -> &WifiReady {
    match state {
        WifiState::Ready(page) => page,
        other => panic!("not ready: {other:?}"),
    }
}

fn world() -> WifiStatus {
    status(
        vec![
            point("Cafe", 80, NetworkLock::Open, NetworkLink::Idle),
            point("Home", 40, NetworkLock::Secured, NetworkLink::Idle),
            point("Home", 70, NetworkLock::Secured, NetworkLink::Idle),
            point("Office", 90, NetworkLock::Secured, NetworkLink::Joined),
            point("Neighbour", 60, NetworkLock::Secured, NetworkLink::Idle),
            point("", 99, NetworkLock::Open, NetworkLink::Idle),
        ],
        &["Home"],
    )
}

#[test]
fn access_points_fold_to_one_network_and_sort_joined_then_remembered_then_signal() {
    let status = world();
    assert_eq!(names(&status), ["Office", "Home", "Cafe", "Neighbour"]);
    let home = status.network(&ssid("Home")).expect("home");
    assert_eq!(
        home.strength,
        SignalStrength(70),
        "the strongest access point"
    );
    assert_eq!(home.remembered, Remembered::Yes);
    assert_eq!(status.joined().map(|n| n.ssid.0.as_str()), Some("Office"));
    assert_eq!(status.others().count(), 3);
}

#[test]
fn a_network_is_joining_if_any_of_its_access_points_is() {
    let status = status(
        vec![
            point("Home", 70, NetworkLock::Secured, NetworkLink::Idle),
            point("Home", 30, NetworkLock::Secured, NetworkLink::Joining),
        ],
        &[],
    );
    assert_eq!(status.networks[0].link, NetworkLink::Joining);
}

#[test]
fn strength_reads_as_one_two_or_three_arcs() {
    const CASES: &[(u8, SignalBars)] = &[
        (0, SignalBars::Weak),
        (33, SignalBars::Weak),
        (34, SignalBars::Fair),
        (66, SignalBars::Fair),
        (67, SignalBars::Good),
        (100, SignalBars::Good),
    ];
    for (percent, want) in CASES {
        assert_eq!(SignalStrength(*percent).signal(), *want, "{percent}");
    }
}

#[test]
fn picking_a_network_joins_at_once_or_asks_for_the_password() {
    let cases: [(&str, &str, Option<WifiEffect>, WifiSheet); 4] = [
        (
            "an open network",
            "Cafe",
            Some(WifiEffect::Join(ssid("Cafe"), None)),
            WifiSheet::Closed,
        ),
        (
            "a remembered one",
            "Home",
            Some(WifiEffect::Join(ssid("Home"), None)),
            WifiSheet::Closed,
        ),
        (
            "a new secured one",
            "Neighbour",
            None,
            WifiSheet::Password(PasswordAsk {
                ssid: ssid("Neighbour"),
                standing: AskStanding::First,
            }),
        ),
        ("the one it is on", "Office", None, WifiSheet::Closed),
    ];
    for (case, name, effect, sheet) in cases {
        let (next, effects) = wifi_step(ready_with(world()), WifiInput::Pick(ssid(name)));
        assert_eq!(effects, effect.into_iter().collect::<Vec<_>>(), "{case}");
        assert_eq!(page_of(&next).sheet, sheet, "{case}");
    }
}

#[test]
fn a_password_closes_the_sheet_and_joins_with_it() {
    let (asked, _) = wifi_step(ready_with(world()), WifiInput::Pick(ssid("Neighbour")));
    let (next, effects) = wifi_step(
        asked,
        WifiInput::Password(ssid("Neighbour"), Passphrase::new("correct horse")),
    );
    assert_eq!(page_of(&next).sheet, WifiSheet::Closed);
    assert_eq!(page_of(&next).joining, Some(ssid("Neighbour")));
    assert_eq!(
        effects,
        [WifiEffect::Join(
            ssid("Neighbour"),
            Some(Passphrase::new("correct horse"))
        )]
    );
}

#[test]
fn a_refused_password_asks_again_and_says_so() {
    let joining = WifiState::Ready(WifiReady {
        joining: Some(ssid("Neighbour")),
        ..page_of(&ready_with(world())).clone()
    });
    let (next, effects) = wifi_step(
        joining,
        WifiInput::Joined(ssid("Neighbour"), Err(JoinFailure::WrongPassword)),
    );
    assert_eq!(effects, []);
    assert_eq!(page_of(&next).joining, None);
    assert_eq!(
        page_of(&next).sheet,
        WifiSheet::Password(PasswordAsk {
            ssid: ssid("Neighbour"),
            standing: AskStanding::Wrong
        })
    );
}

#[test]
fn any_other_failure_is_said_and_the_row_is_free_again() {
    let (joining, _) = wifi_step(ready_with(world()), WifiInput::Pick(ssid("Cafe")));
    let (next, effects) = wifi_step(
        joining,
        WifiInput::Joined(
            ssid("Cafe"),
            Err(JoinFailure::Told("Couldn\u{2019}t join Cafe.".to_owned())),
        ),
    );
    assert_eq!(
        effects,
        [WifiEffect::Tell("Couldn\u{2019}t join Cafe.".to_owned())]
    );
    assert_eq!(page_of(&next).joining, None);
}

#[test]
fn a_second_join_waits_for_the_first() {
    let (joining, _) = wifi_step(ready_with(world()), WifiInput::Pick(ssid("Cafe")));
    let (_, effects) = wifi_step(joining, WifiInput::Pick(ssid("Home")));
    assert_eq!(effects, []);
}

#[test]
fn the_switch_asks_the_service_and_waits_for_it_to_say() {
    let (next, effects) = wifi_step(ready_with(world()), WifiInput::Switch(WifiRadio::Off));
    assert_eq!(effects, [WifiEffect::SetRadio(WifiRadio::Off)]);
    assert_eq!(
        page_of(&next).status.radio,
        WifiRadio::On,
        "not until the service says"
    );
}

#[test]
fn details_open_for_the_network_the_computer_is_on_and_close() {
    let (open, _) = wifi_step(ready_with(world()), WifiInput::OpenDetails);
    assert_eq!(page_of(&open).sheet, WifiSheet::Details(ssid("Office")));
    let (closed, _) = wifi_step(open, WifiInput::CloseSheet);
    assert_eq!(page_of(&closed).sheet, WifiSheet::Closed);
}

#[test]
fn hearing_again_keeps_the_sheet_and_drops_a_join_that_has_ended() {
    let (asked, _) = wifi_step(ready_with(world()), WifiInput::Pick(ssid("Neighbour")));
    let (next, _) = wifi_step(asked, WifiInput::Heard(Ok(world())));
    assert!(matches!(page_of(&next).sheet, WifiSheet::Password(_)));

    let (joining, _) = wifi_step(ready_with(world()), WifiInput::Pick(ssid("Cafe")));
    let now_on = status(
        vec![point("Cafe", 80, NetworkLock::Open, NetworkLink::Joined)],
        &[],
    );
    let (next, _) = wifi_step(joining, WifiInput::Heard(Ok(now_on)));
    assert_eq!(page_of(&next).joining, None);
}

#[test]
fn a_service_that_goes_leaves_the_page_with_nothing_and_says_why() {
    let (next, effects) = wifi_step(
        ready_with(world()),
        WifiInput::Heard(Err(Missing::NoService)),
    );
    assert_eq!(
        (next, effects),
        (WifiState::Unavailable(Missing::NoService), vec![])
    );
}

#[test]
fn a_password_is_long_enough_at_eight_characters_and_never_printed() {
    assert!(!Passphrase::new("seven77").is_long_enough());
    assert!(Passphrase::new("eight888").is_long_enough());
    assert_eq!(
        format!("{:?}", Passphrase::new("hunter2hunter2")),
        "Passphrase(<hidden>)"
    );
}

#[test]
fn the_details_rows_name_the_network_its_security_and_its_signal() {
    let status = world();
    let office = status.joined().expect("office");
    assert_eq!(
        details_rows(office),
        [
            ("Network Name", "Office".to_owned()),
            ("Security", "Password protected".to_owned()),
            ("Signal", "Good (90 percent)".to_owned()),
        ]
    );
}
