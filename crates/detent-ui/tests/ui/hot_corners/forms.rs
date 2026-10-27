//! The forms for Open, App Action and Command, and the app actions offered.

use super::*;

#[test]
fn a_row_for_a_set_of_keys_is_added_once_and_removed_with_its_binding() {
    let sandbox = world("hot-corners-keys", &Fake::default());
    let mut harness = opened(&sandbox);
    pick(&mut harness, "top_left", 1, "Mission Control");

    // Tick Option (the second key) and add the row.
    click(&mut harness, "#hot-corner-add-hot-corners-top-left");
    click(
        &mut harness,
        &format!(
            "{} .hot-corners-keys .ds-checkbox:nth-child(2)",
            cell("top_left")
        ),
    );
    click(&mut harness, "#hot-corner-add-keys-hot-corners-top-left");
    assert_eq!(
        harness.count(&format!("{} .ds-popup", cell("top_left"))),
        2,
        "one pop-up for no keys and one for Option"
    );
    assert_eq!(shows(&harness, "top_left", 2).as_deref(), Some("\u{2013}"));
    assert_eq!(
        stored(&sandbox, "top_left"),
        Some(value("v = [{ action = \"workspaces\" }]")),
        "an empty row stores nothing"
    );

    pick(&mut harness, "top_left", 2, "Lock Screen");
    assert_eq!(
        stored(&sandbox, "top_left"),
        Some(value(
            "v = [{ action = \"workspaces\" }, { action = \"lock\", keys = [\"option\"] }]"
        ))
    );

    // The same keys again are told, not added.
    click(&mut harness, "#hot-corner-add-hot-corners-top-left");
    click(
        &mut harness,
        &format!(
            "{} .hot-corners-keys .ds-checkbox:nth-child(2)",
            cell("top_left")
        ),
    );
    click(&mut harness, "#hot-corner-add-keys-hot-corners-top-left");
    assert!(
        harness
            .html()
            .contains("This corner already has a row for those keys."),
        "{}",
        harness.html()
    );
    assert_eq!(harness.count(&format!("{} .ds-popup", cell("top_left"))), 2);

    click(&mut harness, "[*|aria-label=\"Remove the row for Option\"]");
    assert_eq!(harness.count(&format!("{} .ds-popup", cell("top_left"))), 1);
    assert_eq!(
        stored(&sandbox, "top_left"),
        Some(value("v = [{ action = \"workspaces\" }]"))
    );
}

/// An app with five actions of which two need nothing more.
const MAIL: &str = r#"
vocab = 1
app = "org.quire.Mail"
entities = []

[[actions]]
name = "mail.compose"
label = "Compose"
on = { kind = "nothing" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "offered"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"
params = []

[[actions]]
name = "mail.search"
label = "Find Words"
on = { kind = "nothing" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "offered"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"

[[actions.params]]
name = "words"
label = "Words"
ty = { kind = "text", v = { max = 100, lines = "one" } }
need = { kind = "required" }
sink = "inert"

[[actions]]
name = "mail.reply"
label = "Reply"
on = { kind = "text" }
effect = "read"
classes = []
undo = "not_undoable"
reach = "offered"
latency = "instant"
result = { kind = "nothing" }
keys = { kind = "none" }
lasting = "no"
dry_run = "none"
params = []
"#;

#[test]
fn the_app_action_list_offers_only_what_needs_nothing_more_and_a_pick_stores_it() {
    let sandbox = world("hot-corners-actions", &Fake::with_manifests(&[MAIL]));
    let mut harness = opened(&sandbox);
    pick(&mut harness, "bottom_right", 1, "App Action\u{2026}");
    showing(
        &mut harness,
        "the list of actions",
        "#hot-action-org-quire-mail-compose",
    );
    let html = harness.html();
    assert!(html.contains("Mail"), "grouped under the app:\n{html}");
    for left_out in ["Find Words", "Reply"] {
        assert!(
            !html.contains(&format!(">{left_out}<")),
            "{left_out} needs a parameter or a target:\n{html}"
        );
    }
    click(&mut harness, "#hot-action-org-quire-mail-compose");
    click(&mut harness, "#hot-corner-choose");
    assert_eq!(
        stored(&sandbox, "bottom_right"),
        Some(value(
            "v = [{ action = \"app_action\", app = \"org.quire.Mail\", intent = \"compose\" }]"
        ))
    );
    assert_eq!(
        shows(&harness, "bottom_right", 1).as_deref(),
        Some("Mail \u{203a} Compose")
    );
}

#[test]
fn command_and_open_ask_for_more_and_store_what_was_chosen() {
    let sandbox = world("hot-corners-forms", &Fake::default());
    sandbox.write_application(
        "files",
        "[Desktop Entry]\nType=Application\nName=Files\nExec=files\n",
    );
    let mut harness = opened(&sandbox);

    pick(&mut harness, "top_left", 1, "Command\u{2026}");
    type_into(&mut harness, "#hot-corner-command", "notify-send hi");
    click(&mut harness, "#hot-corner-choose");
    assert_eq!(
        stored(&sandbox, "top_left"),
        Some(value(
            "v = [{ action = \"command\", command = \"notify-send hi\" }]"
        ))
    );

    pick(&mut harness, "top_right", 1, "Open\u{2026}");
    showing(&mut harness, "the apps", "#hot-app-files");
    click(&mut harness, "#hot-app-files");
    click(&mut harness, "#hot-corner-choose");
    assert_eq!(
        stored(&sandbox, "top_right"),
        Some(value("v = [{ action = \"open\", app = \"files\" }]"))
    );
    assert_eq!(
        shows(&harness, "top_right", 1).as_deref(),
        Some("Open Files")
    );
}
