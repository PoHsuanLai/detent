use super::*;
use crate::missing::Missing;

fn addr(last: u8) -> BtAddress {
    BtAddress(format!("AA:BB:CC:DD:EE:{last:02X}"))
}

fn device(last: u8, name: &str, link: BtLink, pairing: BtPairing) -> BtDevice {
    BtDevice {
        address: addr(last),
        name: name.to_owned(),
        kind: BtKind::Audio,
        link,
        pairing,
        battery: None,
    }
}

fn world() -> BtStatus {
    BtStatus::new(
        BtPower::On,
        vec![
            device(1, "Zed Buds", BtLink::Disconnected, BtPairing::Paired),
            device(2, "alpha mouse", BtLink::Connected, BtPairing::Paired),
            device(3, "Beta Speaker", BtLink::Disconnected, BtPairing::Paired),
            device(4, "Nearby Pad", BtLink::Disconnected, BtPairing::Unpaired),
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
    assert_eq!(page_of(&asked).sheet, BtSheet::Forget(addr(2)));
    let (cancelled, effects) = bluetooth_step(asked.clone(), BtInput::Cancel);
    assert_eq!(
        (page_of(&cancelled).sheet.clone(), effects),
        (BtSheet::Closed, vec![])
    );
    let (confirmed, effects) = bluetooth_step(asked, BtInput::ConfirmForget);
    assert_eq!(effects, [BtEffect::Forget(addr(2))]);
    assert_eq!(page_of(&confirmed).sheet, BtSheet::Closed);
}

#[test]
fn a_pairing_question_opens_a_sheet_and_an_answer_goes_to_the_device() {
    let asked = question(PairingAsk::Compare(PairingCode(42)));
    let (open, _) = bluetooth_step(ready(), BtInput::Asked(asked.clone()));
    assert_eq!(page_of(&open).sheet, BtSheet::Pairing(asked));
    let (closed, effects) = bluetooth_step(open, BtInput::Answer(PairingAnswer::Yes));
    assert_eq!(effects, [BtEffect::Answer(PairingAnswer::Yes)]);
    assert_eq!(page_of(&closed).sheet, BtSheet::Closed);
}

#[test]
fn closing_a_pairing_sheet_says_no_to_the_device() {
    let (open, _) = bluetooth_step(ready(), BtInput::Asked(question(PairingAsk::Allow)));
    let (closed, effects) = bluetooth_step(open, BtInput::Cancel);
    assert_eq!(effects, [BtEffect::Answer(PairingAnswer::No)]);
    assert_eq!(page_of(&closed).sheet, BtSheet::Closed);
}

#[test]
fn a_pairing_that_ends_closes_its_own_sheet() {
    let (open, _) = bluetooth_step(
        ready(),
        BtInput::Asked(question(PairingAsk::Show(PairingCode(7)))),
    );
    let (closed, _) = bluetooth_step(open, BtInput::Ended(addr(4), Ok(())));
    assert_eq!(page_of(&closed).sheet, BtSheet::Closed);
}

#[test]
fn the_code_is_six_digits_and_the_sheet_says_which_device() {
    let q = question(PairingAsk::Compare(PairingCode(4_217)));
    assert_eq!(
        pairing_body(&q),
        "Confirm that the number 004217 is shown on \u{201c}Nearby Pad\u{201d}."
    );
    assert!(q.wants_answer());
    assert!(!question(PairingAsk::Show(PairingCode(1))).wants_answer());
    assert_eq!(forget_title("Zed Buds"), "Forget \u{201c}Zed Buds\u{201d}?");
}

#[test]
fn a_service_that_goes_leaves_nothing_and_a_return_brings_the_page_back() {
    let (gone, _) = bluetooth_step(ready(), BtInput::Heard(Err(Missing::NoService)));
    assert_eq!(gone, BtState::Unavailable(Missing::NoService));
    let (back, _) = bluetooth_step(gone, BtInput::Heard(Ok(world())));
    assert!(matches!(back, BtState::Ready(_)));
}

#[test]
fn the_switch_asks_and_waits_for_bluez_to_say() {
    let (next, effects) = bluetooth_step(ready(), BtInput::Switch(BtPower::Off));
    assert_eq!(effects, [BtEffect::SetPower(BtPower::Off)]);
    assert_eq!(page_of(&next).status.power, BtPower::On);
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
fn typing_a_pin_is_answered_to_the_device_and_the_sheet_says_what_to_type() {
    let asked = question(PairingAsk::PinCode);
    assert!(asked.wants_answer());
    assert_eq!(
        pairing_body(&asked),
        "Enter the PIN for \u{201c}Nearby Pad\u{201d}."
    );
    let (open, _) = bluetooth_step(ready(), BtInput::Asked(asked));
    let typed = PairingAnswer::Typed("0000".to_owned());
    let (closed, effects) = bluetooth_step(open, BtInput::Answer(typed.clone()));
    assert_eq!(effects, [BtEffect::Answer(typed)]);
    assert_eq!(page_of(&closed).sheet, BtSheet::Closed);
}

#[test]
fn a_battery_reads_as_a_percentage() {
    assert_eq!(BatteryLevel(80).words(), "80%");
    assert_eq!(BatteryLevel(250).words(), "100%");
}
