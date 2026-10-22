use super::*;

#[test]
fn the_pop_up_lists_the_desktop_then_open_and_app_action_then_command_then_nothing() {
    let labels: Vec<&str> = corner_menu()
        .into_iter()
        .map(|line| match line {
            CornerMenuLine::Choice(choice) => choice.label(),
            CornerMenuLine::Separator => "---",
        })
        .collect();
    assert_eq!(
        labels,
        [
            "Mission Control",
            "Application Windows",
            "Desktop",
            "Notification Centre",
            "Control Centre",
            "Launcher",
            "Assistant",
            "Lock Screen",
            "---",
            "Open\u{2026}",
            "App Action\u{2026}",
            "---",
            "Command\u{2026}",
            "---",
            "\u{2013}",
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
fn a_binding_is_named_in_the_persons_words() {
    // (the binding, the line the pop-up shows, the menu line it marks)
    let cases: Vec<(Option<HotDo>, Option<&str>, Option<CornerChoice>)> = vec![
        (None, None, Some(CornerChoice::Nothing)),
        (
            Some(HotDo::Desk(HotDesk::Lock)),
            None,
            Some(CornerChoice::Desk(HotDesk::Lock)),
        ),
        (
            Some(HotDo::AppAction {
                app: "org.quire.Mail".to_owned(),
                action: "compose".to_owned(),
            }),
            Some("Mail \u{203a} Compose"),
            None,
        ),
        (
            Some(HotDo::AppAction {
                app: "org.quire.Notes".to_owned(),
                action: "new".to_owned(),
            }),
            Some("Notes \u{203a} new"),
            None,
        ),
        (
            Some(HotDo::Open(HotItem::App("org.quire.Mail".to_owned()))),
            Some("Open Mail"),
            None,
        ),
        (
            Some(HotDo::Open(HotItem::Path("~/Documents/".to_owned()))),
            Some("Open Documents"),
            None,
        ),
        (
            Some(HotDo::Open(HotItem::Web(
                "https://example.org:8080/a?b".to_owned(),
            ))),
            Some("Open example.org"),
            None,
        ),
        (
            Some(command("notify-send hi")),
            Some("Command: notify-send hi"),
            None,
        ),
        (
            Some(command("0123456789012345678901234567890123456789")),
            Some("Command: 0123456789012345678901234567\u{2026}"),
            None,
        ),
    ];
    for (does, title, value) in cases {
        let shown = corner_shown(does.as_ref(), &known());
        assert_eq!(shown.title.as_deref(), title, "{does:?}");
        assert_eq!(shown.value, value, "{does:?}");
        if let Some(does) = does {
            assert!(!corner_label(&does, &known()).is_empty());
        }
    }
}
