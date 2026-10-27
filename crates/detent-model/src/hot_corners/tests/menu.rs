use super::*;

#[test]
fn the_pop_up_lists_the_desktop_then_open_and_app_action_then_command_then_nothing() {
    use CornerMenuLine::{Choice, Separator};
    let desk = |desk| Choice(CornerChoice::Desk(desk));
    assert_eq!(
        corner_menu(),
        [
            desk(HotDesk::Workspaces),
            desk(HotDesk::AppWindows),
            desk(HotDesk::ShowDesktop),
            desk(HotDesk::NotificationCenter),
            desk(HotDesk::ControlCenter),
            desk(HotDesk::Launcher),
            desk(HotDesk::Assistant),
            desk(HotDesk::Lock),
            Separator,
            Choice(CornerChoice::Open),
            Choice(CornerChoice::AppAction),
            Separator,
            Choice(CornerChoice::Command),
            Separator,
            Choice(CornerChoice::Nothing),
        ]
    );
}

#[test]
fn a_pick_sets_a_binding_or_asks_for_more() {
    // (the pick, what it comes to)
    let cases: Vec<(CornerChoice, HotPicked)> = vec![
        (
            CornerChoice::Desk(HotDesk::Lock),
            HotPicked::Set(Some(HotDo::Desk(HotDesk::Lock))),
        ),
        (CornerChoice::Nothing, HotPicked::Set(None)),
        (CornerChoice::Open, HotPicked::Ask(CornerAsk::Open)),
        (
            CornerChoice::AppAction,
            HotPicked::Ask(CornerAsk::AppAction),
        ),
        (CornerChoice::Command, HotPicked::Ask(CornerAsk::Command)),
    ];
    for (choice, want) in cases {
        assert_eq!(corner_picked(choice), want, "{choice:?}");
    }
}

#[test]
fn a_desktop_action_or_nothing_marks_its_line_and_any_other_binding_is_said_in_its_own_words() {
    let compose = HotDo::AppAction {
        app: "org.quire.Mail".to_owned(),
        action: "compose".to_owned(),
    };
    let open = HotDo::Open(HotItem::App("org.quire.Mail".to_owned()));
    // (the binding, the line it marks, the binding said in its own words)
    let cases: Vec<(Option<HotDo>, Option<CornerChoice>, Option<HotDo>)> = vec![
        (None, Some(CornerChoice::Nothing), None),
        (Some(lock()), Some(CornerChoice::Desk(HotDesk::Lock)), None),
        (Some(compose.clone()), None, Some(compose)),
        (Some(open.clone()), None, Some(open)),
        (
            Some(command("notify-send hi")),
            None,
            Some(command("notify-send hi")),
        ),
    ];
    for (does, value, says) in cases {
        let shown = corner_shown(does.as_ref());
        assert_eq!(shown.value, value, "{does:?}");
        assert_eq!(shown.says, says, "{does:?}");
    }
}
