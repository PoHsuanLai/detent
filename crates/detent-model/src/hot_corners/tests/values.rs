use super::*;

#[test]
fn a_corners_value_reads_as_its_bindings() {
    use HotModifier::{Alt, Command};
    // (case, the file's value, the bindings read); an older `command` word is folded with its
    // command when the file is read, before a value gets here.
    let cases: Vec<(&str, &str, Vec<HotBinding>)> = vec![
        ("no list is inactive", "v = []", vec![]),
        ("the old word none is inactive", "v = \"none\"", vec![]),
        (
            "the old word lock is one binding with no keys",
            "v = \"lock\"",
            vec![bind(&[], HotDo::Desk(HotDesk::Lock))],
        ),
        (
            "the old word command with no command key is inactive",
            "v = \"command\"",
            vec![],
        ),
        (
            "an unknown old word is inactive",
            "v = \"teleport\"",
            vec![],
        ),
        (
            "a list keeps one binding for each set of keys, in key order",
            "v = [{ keys = [\"option\"], action = \"lock\" }, { action = \"workspaces\" }, { keys = [\"option\"], action = \"launcher\" }]",
            vec![
                bind(&[], HotDo::Desk(HotDesk::Workspaces)),
                bind(&[Alt], HotDo::Desk(HotDesk::Lock)),
            ],
        ),
        (
            "a binding this build cannot read is dropped and the rest stay",
            "v = [{ action = \"teleport\" }, { keys = [\"command\", \"hyper\"], action = \"lock\" }, { keys = [\"command\"], action = \"assistant\" }]",
            vec![bind(&[Command], HotDo::Desk(HotDesk::Assistant))],
        ),
        (
            "an app action carries the app and the action",
            "v = [{ action = \"app_action\", app = \"org.quire.Mail\", intent = \"compose\" }]",
            vec![bind(
                &[],
                HotDo::AppAction {
                    app: "org.quire.Mail".to_owned(),
                    action: "compose".to_owned(),
                },
            )],
        ),
        (
            "an app action without its action is dropped",
            "v = [{ action = \"app_action\", app = \"org.quire.Mail\" }]",
            vec![],
        ),
        (
            "open takes an app, a path or an address",
            "v = [{ action = \"open\", app = \"org.quire.Files\" }, { keys = [\"option\"], action = \"open\", path = \"~/Documents\" }, { keys = [\"command\"], action = \"open\", url = \"https://example.org\" }]",
            vec![
                bind(&[], HotDo::Open(HotItem::App("org.quire.Files".to_owned()))),
                bind(&[Alt], HotDo::Open(HotItem::Path("~/Documents".to_owned()))),
                bind(
                    &[Command],
                    HotDo::Open(HotItem::Web("https://example.org".to_owned())),
                ),
            ],
        ),
        (
            "open with nothing to open is dropped",
            "v = [{ action = \"open\" }]",
            vec![],
        ),
        (
            "a command in a list reads without the old key",
            "v = [{ action = \"command\", command = \"x\" }]",
            vec![bind(&[], command("x"))],
        ),
    ];
    for (case, file, want) in cases {
        assert_eq!(bindings_of(&value(file)), want, "{case}");
    }
}

#[test]
fn bindings_write_the_new_form_and_read_back() {
    use HotModifier::{Alt, Command};
    // (case, the bindings, the TOML the corner stores)
    let cases: Vec<(&str, Vec<HotBinding>, &str)> = vec![
        ("an inactive corner is the empty list", vec![], "v = []"),
        (
            "no keys writes no keys field",
            vec![bind(&[], HotDo::Desk(HotDesk::DisplaySleep))],
            "v = [{ action = \"display_sleep\" }]",
        ),
        (
            "keys are written in glyph order",
            vec![bind(&[Command, Alt], command("x y"))],
            "v = [{ action = \"command\", command = \"x y\", keys = [\"option\", \"command\"] }]",
        ),
        (
            "an app action writes the app and the action",
            vec![bind(
                &[Alt],
                HotDo::AppAction {
                    app: "org.quire.Mail".to_owned(),
                    action: "compose".to_owned(),
                },
            )],
            "v = [{ action = \"app_action\", app = \"org.quire.Mail\", intent = \"compose\", keys = [\"option\"] }]",
        ),
        (
            "an address is written as url",
            vec![bind(
                &[],
                HotDo::Open(HotItem::Web("https://example.org".to_owned())),
            )],
            "v = [{ action = \"open\", url = \"https://example.org\" }]",
        ),
    ];
    for (case, bindings, text) in cases {
        let written = corner_value(&bindings);
        assert_eq!(written, value(text), "{case}");
        assert_eq!(bindings_of(&written), bindings, "{case}: read back");
    }
}

#[test]
fn a_corner_key_holds_the_old_words_and_lists_of_readable_rows_only() {
    // (case, the value, held)
    let cases: &[(&str, &str, bool)] = &[
        ("an old word", "v = \"lock\"", true),
        ("none", "v = \"none\"", true),
        ("command", "v = \"command\"", true),
        ("an unknown word", "v = \"teleport\"", false),
        ("the empty list", "v = []", true),
        ("a readable list", "v = [{ action = \"lock\" }]", true),
        (
            "a list with an unreadable row",
            "v = [{ action = \"lock\" }, { action = \"x\" }]",
            false,
        ),
        ("a number", "v = 3", false),
    ];
    for (case, file, held) in cases {
        assert_eq!(corner_admits(&value(file)), *held, "{case}");
    }
}

#[test]
fn corners_know_their_keys() {
    for corner in Corner::ALL {
        assert_eq!(Corner::of_path(corner.path()), Some(corner));
    }
    assert_eq!(Corner::of_path("hot_corners.dwell_ms"), None);
}
