//! The Hot Corners sheet's words: the corners, the keys, what each pick is called and what a
//! binding says it does.

use detent_model::{
    Corner, CornerChoice, HotAddFault, HotDesk, HotDo, HotItem, HotKeys, HotKnown, HotModifier,
    HotOpenKind,
};

/// The longest command a pop-up button spells out.
const COMMAND_SHOWN: usize = 28;

/// What names the corner to a person who cannot see the sheet's picture.
pub(crate) fn corner_name(corner: Corner) -> &'static str {
    match corner {
        Corner::TopLeft => "Top left",
        Corner::TopRight => "Top right",
        Corner::BottomLeft => "Bottom left",
        Corner::BottomRight => "Bottom right",
    }
}

/// What the chooser calls a kind of thing to open.
pub(crate) fn open_kind_label(kind: HotOpenKind) -> &'static str {
    match kind {
        HotOpenKind::App => "App",
        HotOpenKind::File => "File",
        HotOpenKind::Folder => "Folder",
        HotOpenKind::Web => "Web Address",
    }
}

/// What a person who cannot see the glyph hears.
pub(crate) fn modifier_name(modifier: HotModifier) -> &'static str {
    match modifier {
        HotModifier::Control => "Control",
        HotModifier::Alt => "Option",
        HotModifier::Shift => "Shift",
        HotModifier::Command => "Command",
    }
}

/// The glyph a menu or a row writes.
pub(crate) fn modifier_glyph(modifier: HotModifier) -> &'static str {
    match modifier {
        HotModifier::Control => "\u{2303}",
        HotModifier::Alt => "\u{2325}",
        HotModifier::Shift => "\u{21e7}",
        HotModifier::Command => "\u{2318}",
    }
}

/// The glyphs a row writes ("⌥⌘"), empty for the empty set.
pub(crate) fn keys_glyphs(keys: &HotKeys) -> String {
    keys.modifiers().into_iter().map(modifier_glyph).collect()
}

/// The names a person who cannot see the glyphs hears ("Option Command").
pub(crate) fn keys_names(keys: &HotKeys) -> String {
    keys.modifiers()
        .into_iter()
        .map(modifier_name)
        .collect::<Vec<_>>()
        .join(" ")
}

/// What a desktop action is called.
pub(crate) fn desk_label(desk: HotDesk) -> &'static str {
    match desk {
        HotDesk::Workspaces => "Mission Control",
        HotDesk::AppWindows => "Application Windows",
        HotDesk::ShowDesktop => "Desktop",
        HotDesk::NotificationCenter => "Notification Centre",
        HotDesk::ControlCenter => "Control Centre",
        HotDesk::Launcher => "Launcher",
        HotDesk::Assistant => "Assistant",
        HotDesk::Lock => "Lock Screen",
        HotDesk::DisplaySleep => "Put Display to Sleep",
    }
}

/// What the menu calls a choice. A line that asks for more ends in an ellipsis.
pub(crate) fn choice_label(choice: CornerChoice) -> &'static str {
    match choice {
        CornerChoice::Desk(desk) => desk_label(desk),
        CornerChoice::Open => "Open\u{2026}",
        CornerChoice::AppAction => "App Action\u{2026}",
        CornerChoice::Command => "Command\u{2026}",
        CornerChoice::Nothing => "\u{2013}",
    }
}

/// The sentence that says why nothing was added.
pub(crate) fn add_fault(fault: HotAddFault) -> &'static str {
    match fault {
        HotAddFault::NoKeys => "Tick at least one key.",
        HotAddFault::Already => "This corner already has a row for those keys.",
    }
}

/// The last name in `path`, without a trailing slash; the path itself when it has none.
fn name_of_path(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
}

/// The host of a web address, without its scheme, path or port.
fn host_of(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    rest.split(['/', '?', '#'])
        .next()
        .and_then(|authority| authority.rsplit('@').next())
        .and_then(|host| host.split(':').next())
        .filter(|host| !host.is_empty())
        .unwrap_or(url)
}

/// `text` cut to `COMMAND_SHOWN` characters, with an ellipsis when it was cut.
fn short(text: &str) -> String {
    let line = text.lines().next().unwrap_or_default().trim();
    match line.char_indices().nth(COMMAND_SHOWN) {
        Some((at, _)) => format!("{}\u{2026}", &line[..at]),
        None => line.to_owned(),
    }
}

/// What `does` is called where a person reads it: "Open Files", "Mail › Compose",
/// "Command: notify-send hi".
pub(crate) fn corner_label(does: &HotDo, known: &HotKnown) -> String {
    match does {
        HotDo::Desk(desk) => desk_label(*desk).to_owned(),
        HotDo::AppAction { app, action } => known.action_line(app, action),
        HotDo::Open(HotItem::App(id)) => format!("Open {}", known.app_title(id)),
        HotDo::Open(HotItem::Path(path)) => format!("Open {}", name_of_path(path)),
        HotDo::Open(HotItem::Web(url)) => format!("Open {}", host_of(url)),
        HotDo::Command(command) => format!("Command: {}", short(command)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::{CornerMenuLine, HotActionChoice, HotApp, corner_menu};

    fn known() -> HotKnown {
        HotKnown {
            apps: vec![HotApp {
                id: "org.quire.Mail".to_owned(),
                name: "Mail".to_owned(),
            }],
            actions: vec![HotActionChoice {
                app: "org.quire.Mail".to_owned(),
                action: "compose".to_owned(),
                label: "Compose".to_owned(),
            }],
        }
    }

    fn command(text: &str) -> HotDo {
        HotDo::Command(text.to_owned())
    }

    #[test]
    fn the_pop_up_lists_the_desktop_then_open_and_app_action_then_command_then_nothing() {
        let labels: Vec<&str> = corner_menu()
            .into_iter()
            .map(|line| match line {
                CornerMenuLine::Choice(choice) => choice_label(choice),
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
    fn a_binding_is_named_in_the_persons_words() {
        let cases = [
            (
                HotDo::AppAction {
                    app: "org.quire.Mail".to_owned(),
                    action: "compose".to_owned(),
                },
                "Mail \u{203a} Compose",
            ),
            (
                HotDo::AppAction {
                    app: "org.quire.Notes".to_owned(),
                    action: "new".to_owned(),
                },
                "Notes \u{203a} new",
            ),
            (
                HotDo::Open(HotItem::App("org.quire.Mail".to_owned())),
                "Open Mail",
            ),
            (
                HotDo::Open(HotItem::Path("~/Documents/".to_owned())),
                "Open Documents",
            ),
            (
                HotDo::Open(HotItem::Web("https://example.org:8080/a?b".to_owned())),
                "Open example.org",
            ),
            (command("notify-send hi"), "Command: notify-send hi"),
            (
                command("0123456789012345678901234567890123456789"),
                "Command: 0123456789012345678901234567\u{2026}",
            ),
            (HotDo::Desk(HotDesk::Lock), "Lock Screen"),
        ];
        for (does, want) in cases {
            assert_eq!(corner_label(&does, &known()), want, "{does:?}");
        }
    }

    #[test]
    fn keys_are_written_as_glyphs_and_heard_as_names() {
        let both = HotKeys::of([HotModifier::Alt, HotModifier::Command]);
        assert_eq!(keys_glyphs(&both), "\u{2325}\u{2318}");
        assert_eq!(keys_names(&both), "Option Command");
        let all = HotKeys::of(HotModifier::ALL);
        assert_eq!(keys_glyphs(&all), "\u{2303}\u{2325}\u{21e7}\u{2318}");
        assert_eq!(keys_glyphs(&HotKeys::default()), "");
    }

    #[test]
    fn a_corner_is_named_for_a_person_who_cannot_see_the_picture() {
        let names: Vec<&str> = Corner::ALL.into_iter().map(corner_name).collect();
        assert_eq!(
            names,
            ["Top left", "Top right", "Bottom left", "Bottom right"]
        );
    }

    #[test]
    fn a_kind_a_modifier_and_a_fault_have_their_words() {
        let kinds: Vec<&str> = HotOpenKind::ALL.into_iter().map(open_kind_label).collect();
        assert_eq!(kinds, ["App", "File", "Folder", "Web Address"]);
        assert_eq!(modifier_name(HotModifier::Alt), "Option");
        assert_eq!(add_fault(HotAddFault::NoKeys), "Tick at least one key.");
        assert_eq!(
            add_fault(HotAddFault::Already),
            "This corner already has a row for those keys."
        );
    }
}
