use super::*;
use crate::Absence;

fn addr(last: u8) -> BtAddress {
    BtAddress(format!("AA:BB:CC:DD:EE:{last:02X}"))
}

fn device(last: u8, name: &str, link: BtLink, paired: bool) -> BtDevice {
    BtDevice {
        address: addr(last),
        name: name.to_owned(),
        kind: BtKind::Audio,
        link,
        paired,
        battery: None,
    }
}

fn world() -> BtStatus {
    BtStatus::new(
        true,
        vec![
            device(1, "Zed Buds", BtLink::Disconnected, true),
            device(2, "alpha mouse", BtLink::Connected, true),
            device(3, "Beta Speaker", BtLink::Disconnected, true),
            device(4, "Nearby Pad", BtLink::Disconnected, false),
        ],
    )
}

fn ready() -> BtState {
    let (state, _) = bluetooth_step(BtState::Looking, BtInput::Heard(Ok(world())));
    state
}

fn page_of(state: &BtState) -> &BtReady {
    match state {
        BtState::Ready(page) => page,
        other => panic!("not ready: {other:?}"),
    }
}

fn question(ask: PairingAsk) -> PairingQuestion {
    PairingQuestion {
        address: addr(4),
        name: "Nearby Pad".to_owned(),
        ask,
    }
}

#[test]
fn devices_split_into_mine_and_nearby_with_the_connected_first() {
    let status = world();
    let mine: Vec<&str> = status.mine().map(|d| d.name.as_str()).collect();
    assert_eq!(mine, ["alpha mouse", "Beta Speaker", "Zed Buds"]);
    let nearby: Vec<&str> = status.nearby().map(|d| d.name.as_str()).collect();
    assert_eq!(nearby, ["Nearby Pad"]);
}

#[test]
fn an_icon_name_says_what_a_device_is() {
    const CASES: &[(&str, BtKind)] = &[
        ("audio-headset", BtKind::Audio),
        ("input-mouse", BtKind::Mouse),
        ("input-keyboard", BtKind::Keyboard),
        ("input-gaming", BtKind::Gamepad),
        ("phone", BtKind::Phone),
        ("", BtKind::Other),
    ];
    for (icon, want) in CASES {
        assert_eq!(BtKind::of_icon(icon), *want, "{icon:?}");
    }
}

#[test]
fn connecting_and_pairing_mark_the_device_busy_and_ask_once() {
    let (next, effects) = bluetooth_step(ready(), BtInput::Connect(addr(1)));
    assert_eq!(effects, [BtEffect::Connect(addr(1))]);
    assert!(page_of(&next).is_working(&addr(1)));
    let (_, again) = bluetooth_step(next.clone(), BtInput::Pair(addr(4)));
    assert_eq!(again, [], "one at a time");
    let (done, _) = bluetooth_step(next, BtInput::Ended(addr(1), Ok(())));
    assert_eq!(page_of(&done).busy, None);
    let (_, pair) = bluetooth_step(done, BtInput::Pair(addr(4)));
    assert_eq!(pair, [BtEffect::Pair(addr(4))]);
}

#[test]
fn a_failure_is_said_and_frees_the_device() {
    let (busy, _) = bluetooth_step(ready(), BtInput::Pair(addr(4)));
    let (next, effects) = bluetooth_step(
        busy,
        BtInput::Ended(addr(4), Err("Couldn\u{2019}t pair.".to_owned())),
    );
    assert_eq!(
        effects,
        [BtEffect::Tell("Couldn\u{2019}t pair.".to_owned())]
    );
    assert_eq!(page_of(&next).busy, None);
}

#[test]
fn forgetting_asks_first_and_only_confirming_forgets() {
    let (asked, effects) = bluetooth_step(ready(), BtInput::AskForget(addr(2)));
    assert_eq!(effects, []);
    assert_eq!(page_of(&asked).asking, BtAsking::Forget(addr(2)));
    let (cancelled, effects) = bluetooth_step(asked.clone(), BtInput::Cancel);
    assert_eq!(
        (page_of(&cancelled).asking.clone(), effects),
        (BtAsking::Nothing, vec![])
    );
    let (confirmed, effects) = bluetooth_step(asked, BtInput::ConfirmForget);
    assert_eq!(effects, [BtEffect::Forget(addr(2))]);
    assert_eq!(page_of(&confirmed).asking, BtAsking::Nothing);
}

#[test]
fn a_pairing_question_opens_a_sheet_and_an_answer_goes_to_the_device() {
    let asked = question(PairingAsk::Compare(PairingCode(42)));
    let (open, _) = bluetooth_step(ready(), BtInput::Asked(asked.clone()));
    assert_eq!(page_of(&open).asking, BtAsking::Pairing(asked));
    let (closed, effects) = bluetooth_step(open, BtInput::Answer(PairingAnswer::Yes));
    assert_eq!(effects, [BtEffect::Answer(PairingAnswer::Yes)]);
    assert_eq!(page_of(&closed).asking, BtAsking::Nothing);
}

#[test]
fn closing_a_pairing_sheet_says_no_to_the_device() {
    let (open, _) = bluetooth_step(ready(), BtInput::Asked(question(PairingAsk::Allow)));
    let (closed, effects) = bluetooth_step(open, BtInput::Cancel);
    assert_eq!(effects, [BtEffect::Answer(PairingAnswer::No)]);
    assert_eq!(page_of(&closed).asking, BtAsking::Nothing);
}

#[test]
fn a_pairing_that_ends_closes_its_own_sheet() {
    let (open, _) = bluetooth_step(
        ready(),
        BtInput::Asked(question(PairingAsk::Show(PairingCode(7)))),
    );
    let (closed, _) = bluetooth_step(open, BtInput::Ended(addr(4), Ok(())));
    assert_eq!(page_of(&closed).asking, BtAsking::Nothing);
}

#[test]
fn only_a_number_to_type_on_the_device_wants_no_answer() {
    let q = question(PairingAsk::Compare(PairingCode(4_217)));
    assert!(q.wants_answer());
    assert!(!question(PairingAsk::Show(PairingCode(1))).wants_answer());
}

#[test]
fn a_service_that_goes_leaves_nothing_and_a_return_brings_the_page_back() {
    let (gone, _) = bluetooth_step(ready(), BtInput::Heard(Err(Absence::NotRunning)));
    assert_eq!(gone, BtState::Unavailable(Absence::NotRunning));
    let (back, _) = bluetooth_step(gone, BtInput::Heard(Ok(world())));
    assert!(matches!(back, BtState::Ready(_)));
}

#[test]
fn the_switch_asks_and_waits_for_bluez_to_say() {
    let (next, effects) = bluetooth_step(ready(), BtInput::Switch(false));
    assert_eq!(effects, [BtEffect::SetPower(false)]);
    assert!(page_of(&next).status.powered);
}

#[test]
fn a_pin_is_one_to_sixteen_characters_and_a_passkey_up_to_six_digits() {
    let cases = [
        ("pin", PairingAsk::PinCode, "0000", true),
        ("pin with letters", PairingAsk::PinCode, "ab12", true),
        ("empty pin", PairingAsk::PinCode, "", false),
        ("long pin", PairingAsk::PinCode, "12345678901234567", false),
        ("passkey", PairingAsk::Passkey, "004217", true),
        ("passkey letters", PairingAsk::Passkey, "12ab", false),
        ("long passkey", PairingAsk::Passkey, "1234567", false),
        ("a number to compare", PairingAsk::Allow, "1234", false),
    ];
    for (case, ask, text, want) in cases {
        assert_eq!(ask.accepts(text), want, "{case}");
    }
}

#[test]
fn typing_a_pin_is_answered_to_the_device() {
    let asked = question(PairingAsk::PinCode);
    assert!(asked.wants_answer());
    let (open, _) = bluetooth_step(ready(), BtInput::Asked(asked));
    let typed = PairingAnswer::Typed("0000".to_owned());
    let (closed, effects) = bluetooth_step(open, BtInput::Answer(typed.clone()));
    assert_eq!(effects, [BtEffect::Answer(typed)]);
    assert_eq!(page_of(&closed).asking, BtAsking::Nothing);
}
