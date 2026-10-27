//! The rows read into programs and powers.

use super::*;

#[test]
fn the_powers_are_in_the_pages_order_and_each_has_its_word() {
    let words: Vec<&str> = PrivacyPower::ALL.iter().map(|power| power.word()).collect();
    assert_eq!(
        words,
        [
            "screen",
            "keys",
            "typing_help",
            "clipboard",
            "lock",
            "displays",
            "windows"
        ]
    );
    for power in PrivacyPower::ALL {
        assert_eq!(PrivacyPower::of_word(power.word()), Some(power));
    }
    assert_eq!(PrivacyPower::of_word("teleport"), None);
}

#[test]
fn rows_become_programs_with_every_power_and_the_unanswered_ones_ask() {
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .answered("clipboard", "bbb", GRIM, "off")
        .answered("windows", "ccc", LOUPE, "on")
        .empty("keys")
        .empty("lock")
        .pane();
    let who: Vec<&str> = shown
        .programs
        .iter()
        .map(|p| p.subject.0.as_str())
        .collect();
    assert_eq!(who, [GRIM, LOUPE], "in the order the rows first name them");
    assert_eq!(shown.programs[0].cells.len(), 7);
    // `(program, power, where it stands)`.
    const CASES: &[(&str, PrivacyPower, PowerStanding)] = &[
        (GRIM, PrivacyPower::ScreenRecording, PowerStanding::Allowed),
        (GRIM, PrivacyPower::Clipboard, PowerStanding::Denied),
        (GRIM, PrivacyPower::KeyboardInput, PowerStanding::Ask),
        (LOUPE, PrivacyPower::Windows, PowerStanding::Allowed),
        (LOUPE, PrivacyPower::ScreenRecording, PowerStanding::Ask),
    ];
    for (who, power, want) in CASES {
        assert_eq!(standing(&shown, who, *power), *want, "{who} {power:?}");
    }
    assert_eq!(
        Publication::default().empty("screen").pane(),
        PrivacyPane::default(),
        "a section nobody asked in lists no program"
    );
}

#[test]
fn an_administrators_answer_is_locked_and_wins_over_the_persons() {
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .fixed("screen", "bbb", GRIM, "off")
        .fixed("keys", "ccc", GRIM, "on")
        .pane();
    let grim = &shown.programs[0];
    let screen = grim.cell(PrivacyPower::ScreenRecording).expect("a cell");
    assert!(screen.locked() && !screen.editable());
    assert_eq!(
        screen.standing,
        PowerStanding::Denied,
        "the floor, not the person"
    );
    assert!(
        grim.cell(PrivacyPower::KeyboardInput)
            .expect("a cell")
            .locked()
    );
    let clipboard = grim.cell(PrivacyPower::Clipboard).expect("a cell");
    assert!(
        !clipboard.locked() && !clipboard.editable(),
        "nobody has answered"
    );
}

#[test]
fn a_power_nobody_answered_is_a_switch_at_ask_with_nothing_to_forget() {
    // Grim has answered one power; Loupe none, so it is listed only because of its Ask rows.
    let shown = Publication::default()
        .answered("screen", "aaa", GRIM, "on")
        .unanswered("keys", "bbb", GRIM)
        .unanswered("screen", "ccc", LOUPE)
        .unanswered("keys", "ddd", LOUPE)
        .pane();
    let who: Vec<&str> = shown
        .programs
        .iter()
        .map(|p| p.subject.0.as_str())
        .collect();
    assert_eq!(who, [GRIM, LOUPE]);
    // `(program, power, the switch)`.
    const CASES: &[(&str, PrivacyPower, &str)] = &[
        (GRIM, PrivacyPower::KeyboardInput, "privacy.keys.bbb"),
        (LOUPE, PrivacyPower::ScreenRecording, "privacy.screen.ccc"),
        (LOUPE, PrivacyPower::KeyboardInput, "privacy.keys.ddd"),
    ];
    for (who, power, switch) in CASES {
        let cell = shown
            .programs
            .iter()
            .find(|program| program.subject.0 == *who)
            .and_then(|program| program.cell(*power))
            .expect("a cell");
        assert_eq!(cell.standing, PowerStanding::Ask, "{who} {power:?}");
        assert_eq!(
            cell.kind,
            CellKind::Remembered {
                switch: path(switch),
                forget: None,
            },
            "{who} {power:?}"
        );
        assert!(cell.editable() && !cell.locked(), "{who} {power:?}");
    }
    // The answered one beside it keeps its own action.
    assert_eq!(
        standing(&shown, GRIM, PrivacyPower::ScreenRecording),
        PowerStanding::Allowed
    );
}

#[test]
fn a_row_that_is_not_an_answer_is_left_out() {
    let mut shown = Publication::default().answered("screen", "aaa", GRIM, "on");
    // An administrator's read-out cannot say `ask`: the floor is Allowed or Denied.
    shown.keys.push(spec(
        "privacy.keys.ggg",
        GRIM,
        KeyKind::Fixed {
            variant: "ask".to_owned(),
        },
    ));
    shown
        .values
        .insert(path("privacy.keys.ggg"), "ask".to_owned());
    // A word that is neither on nor off, a power this page does not know, an id with a dot.
    shown
        .keys
        .push(spec("privacy.screen.ddd", GRIM, KeyKind::Text));
    shown
        .keys
        .push(spec("privacy.teleport.eee", "/bin/x", KeyKind::Text));
    shown
        .keys
        .push(spec("privacy.screen.f.g", "/bin/y", KeyKind::Text));
    shown
        .values
        .insert(path("privacy.screen.ddd"), "maybe".to_owned());
    let pane = shown.pane();
    assert_eq!(pane.programs.len(), 1);
    assert_eq!(
        standing(&pane, GRIM, PrivacyPower::ScreenRecording),
        PowerStanding::Allowed
    );
    let keys = pane.programs[0]
        .cell(PrivacyPower::KeyboardInput)
        .expect("a cell");
    assert_eq!(
        keys.kind,
        CellKind::Asking,
        "the read-out at ask is not an answer"
    );
}
