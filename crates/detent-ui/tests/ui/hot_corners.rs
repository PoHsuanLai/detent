//! The Desktop & Dock page's "Hot Corners…" button and its sheet, over a stand-in schema whose
//! corner keys ask for the sheet, a fake list of the apps' actions and a fake chooser.

use crate::support::Sandbox;
use crate::support::corners::Fake;
use crate::support::drive::{click, page_of, showing, type_into};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Harness;
use ds_settings::schema::{
    AppId, ControlName, Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page, Schema,
    Section,
};

const FILE: &str = "sill/settings.toml";

/// One corner key of the stand-in schema, as quire declares it: text, drawn by the `corners`
/// control, empty at first.
fn corner_key(path: &str) -> KeySpec {
    let (kind, default, control) = (
        KeyKind::Text,
        toml::Value::Array(Vec::new()),
        Some(ControlName("corners".to_owned())),
    );
    KeySpec {
        control,
        path: KeyPath(path.to_owned()),
        kind,
        default,
        label: Label(path.to_owned()),
        help: Help("What the corner does.".to_owned()),
        page: Page::Dock,
        section: Section("Hot Corners".to_owned()),
        exposure: Exposure::Advanced,
        labels: Default::default(),
        agent: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
    }
}

fn sill() -> Schema {
    let corners = ["top_left", "top_right", "bottom_left", "bottom_right"];
    let keys: Vec<KeySpec> = corners
        .iter()
        .map(|corner| corner_key(&format!("hot_corners.{corner}")))
        .collect();
    Schema {
        pages: Vec::new(),
        app: AppId("sill".to_owned()),
        file: FilePath(FILE.to_owned()),
        foreign: Default::default(),
        version: 1,
        key: keys,
    }
}

fn world(name: &str, fake: &Fake) -> Sandbox {
    let sandbox = Sandbox::new(name).with_corners(fake.desk());
    sandbox.remove_schema("sill");
    sandbox.install_schema(&sill());
    sandbox
}

fn opening() -> Opening {
    Opening {
        page: Some(PageSlug("dock".to_owned())),
        query: String::new(),
    }
}

/// The page, with the sheet open.
fn opened(sandbox: &Sandbox) -> Harness {
    let mut harness = sandbox.harness(opening());
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
    click(&mut harness, "#hot-corners-open");
    showing(&mut harness, "the sheet", ".hot-corners-screen");
    harness
}

/// The cell of the corner `name` (`top_left`).
fn cell(name: &str) -> String {
    format!(".hot-corners-cell[*|data-corner=\"hot_corners.{name}\"]")
}

/// The pop-up button of row `row` (from 1) of the corner `name`.
fn pop_up(name: &str, row: usize) -> String {
    format!(
        "{} .hot-corners-row:nth-child({row}) .ds-popup .ds-button",
        cell(name)
    )
}

/// Opens the pop-up of row `row` of the corner `name` and picks the line called `line`. The lines
/// are found by their words, so a rule between groups does not move them.
fn pick(harness: &mut Harness, name: &str, row: usize, line: &str) {
    click(harness, &pop_up(name, row));
    let found = (1..=20)
        .map(|n| format!(".ds-menu-item:nth-child({n}) .ds-menu-label"))
        .find(|selector| harness.text_of(selector).as_deref() == Some(line))
        .unwrap_or_else(|| panic!("no line called {line}:\n{}", harness.html()));
    click(harness, &found);
}

/// What the pop-up of row `row` of the corner `name` says.
fn shows(harness: &Harness, name: &str, row: usize) -> Option<String> {
    harness.text_of(&format!(
        "{} .hot-corners-row:nth-child({row}) .ds-popup .ds-button-label",
        cell(name)
    ))
}

/// The value the file holds for the corner `name`.
fn stored(sandbox: &Sandbox, name: &str) -> Option<toml::Value> {
    let table: toml::Table = sandbox.read(FILE)?.parse().ok()?;
    table.get("hot_corners")?.get(name).cloned()
}

/// The value of the one key `v` in the TOML `text`.
fn value(text: &str) -> toml::Value {
    let table: toml::Table = text.parse().expect("test TOML");
    table["v"].clone()
}

#[test]
fn the_dock_page_ends_in_a_button_and_the_corners_have_no_rows_of_their_own() {
    let sandbox = world("hot-corners-button", &Fake::default());
    let harness = sandbox.harness(opening());
    assert_eq!(harness.count("#hot-corners-open"), 1);
    assert_eq!(
        harness.count(".hot-corners-screen"),
        0,
        "the sheet is closed"
    );
    let html = harness.html();
    assert!(html.contains("Hot Corners\u{2026}"), "{html}");
    assert!(
        !html.contains("hot_corners.top_left"),
        "neither the corners nor their old command keys have a row:\n{html}"
    );
}

#[test]
fn each_pick_writes_its_corner_in_the_new_form() {
    let sandbox = world("hot-corners-picks", &Fake::default());
    let mut harness = opened(&sandbox);
    assert_eq!(harness.count(".hot-corners-cell"), 4);
    assert_eq!(sandbox.read(FILE), None, "looking writes nothing");

    // (corner, the line picked, what the corner stores, what its pop-up shows after)
    let cases: &[(&str, &str, &str, &str)] = &[
        (
            "top_left",
            "Mission Control",
            "v = [{ action = \"workspaces\" }]",
            "Mission Control",
        ),
        (
            "top_right",
            "Lock Screen",
            "v = [{ action = \"lock\" }]",
            "Lock Screen",
        ),
        (
            "bottom_left",
            "Notification Centre",
            "v = [{ action = \"notification_center\" }]",
            "Notification Centre",
        ),
        (
            "bottom_right",
            "Assistant",
            "v = [{ action = \"assistant\" }]",
            "Assistant",
        ),
        ("top_left", "\u{2013}", "v = []", "\u{2013}"),
    ];
    for (corner, line, stores, shown) in cases {
        pick(&mut harness, corner, 1, line);
        assert_eq!(
            stored(&sandbox, corner),
            Some(value(stores)),
            "{corner}: {line}"
        );
        assert_eq!(
            shows(&harness, corner, 1).as_deref(),
            Some(*shown),
            "{corner}"
        );
    }
    let text = sandbox.read(FILE).expect("the file was written");
    assert!(
        !text.contains("_command"),
        "the old command keys are not written back:\n{text}"
    );
}

#[test]
fn a_file_written_before_the_lists_shows_as_it_was_and_is_rewritten_only_where_changed() {
    let sandbox = world("hot-corners-legacy", &Fake::default());
    let before = "[hot_corners]\ntop_left = \"lock\"\ntop_right = \"command\"\ntop_right_command = \"notify-send hi\"\nbottom_left = \"none\"\n";
    sandbox.write(FILE, before);
    let mut harness = opened(&sandbox);
    // (corner, what its pop-up shows)
    let cases = [
        ("top_left", "Lock Screen"),
        ("top_right", "Command: notify-send hi"),
        ("bottom_left", "\u{2013}"),
        ("bottom_right", "\u{2013}"),
    ];
    for (corner, shown) in cases {
        assert_eq!(
            shows(&harness, corner, 1).as_deref(),
            Some(shown),
            "{corner}"
        );
    }
    assert_eq!(
        sandbox.read(FILE).as_deref(),
        Some(before),
        "showing writes nothing"
    );

    pick(&mut harness, "bottom_left", 1, "Launcher");
    let text = sandbox.read(FILE).expect("the file");
    assert_eq!(
        stored(&sandbox, "bottom_left"),
        Some(value("v = [{ action = \"launcher\" }]")),
        "{text}"
    );
    assert!(
        text.contains("top_left = \"lock\"") && text.contains("top_right = \"command\""),
        "the corners that were not changed keep what they held:\n{text}"
    );
}

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
