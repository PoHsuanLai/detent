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

mod forms;
mod sheet;

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
        ends: None,
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
        footnotes: Vec::new(),
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
/// are found by their titles, so a rule between groups does not move them.
fn pick(harness: &mut Harness, name: &str, row: usize, line: &str) {
    click(harness, &pop_up(name, row));
    let found = format!(".ds-menu-item[aria-label=\"{line}\"] .ds-menu-label");
    assert_eq!(
        harness.count(&found),
        1,
        "no line called {line}:\n{}",
        harness.html()
    );
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
