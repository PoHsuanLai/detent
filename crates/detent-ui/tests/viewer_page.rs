//! The Viewer page: anyview's `viewer.*` keys, drawn when anyview's schema is installed.
//!
//! anyview does not write its schema yet (its FINDINGS wait for quire's `viewer.*` rows), so
//! the test installs a scratch schema of the keys its FINDINGS name, on `Page::Viewer`.

mod support;

use detent_files::load;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Query;
use ds_settings::schema::{
    AppId, Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page, Schema, Section,
};
use support::Sandbox;
use support::drive::{click, page_of, title_of};

const FILE: &str = "anyview/settings.toml";

fn key(path: &str, label: &str, section: &str, kind: KeyKind, default: toml::Value) -> KeySpec {
    key_in(path, label, section, kind, default, Exposure::Basic)
}

fn key_in(
    path: &str,
    label: &str,
    section: &str,
    kind: KeyKind,
    default: toml::Value,
    exposure: Exposure,
) -> KeySpec {
    KeySpec {
        path: KeyPath(path.to_owned()),
        kind,
        default,
        label: Label(label.to_owned()),
        help: Help(format!("{label}, as the viewer reads it.")),
        page: Page::Viewer,
        section: Section(section.to_owned()),
        exposure,
        labels: Default::default(),
        agent: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
    }
}

fn bounded(min: i64, max: i64, unit: &str) -> KeyKind {
    KeyKind::Bounded {
        min,
        max,
        unit: Some(unit.to_owned()),
    }
}

/// A scratch stand-in for anyview's schema.
fn anyview() -> Schema {
    Schema {
        app: AppId("anyview".to_owned()),
        file: FilePath(FILE.to_owned()),
        version: 1,
        key: vec![
            key(
                "viewer.history.enabled",
                "Keep history",
                "History",
                KeyKind::Toggle {
                    variants: ["on".to_owned(), "off".to_owned()],
                },
                toml::Value::String("on".to_owned()),
            ),
            key(
                "viewer.chrome.hide_after_ms",
                "Hide controls after",
                "Controls",
                bounded(500, 10_000, "ms"),
                toml::Value::Integer(2_500),
            ),
            key_in(
                "viewer.zoom.step_percent",
                "Zoom step",
                "Zoom",
                bounded(5, 100, "%"),
                toml::Value::Integer(25),
                Exposure::Advanced,
            ),
        ],
    }
}

fn opening(page: &str) -> Opening {
    Opening {
        page: Some(PageSlug(page.to_owned())),
        query: String::new(),
    }
}

#[test]
fn without_anyviews_schema_there_is_no_viewer_page() {
    let sandbox = Sandbox::new("viewer-absent");
    let harness = sandbox.harness(Opening::default());
    let html = harness.html();
    assert!(!html.contains("Viewer"), "no schema, no page");
}

#[test]
fn the_viewer_page_draws_anyviews_keys_under_its_sections() {
    let sandbox = Sandbox::new("viewer-page");
    sandbox.install_schema(&anyview());
    let harness = sandbox.harness(opening("viewer"));
    assert_eq!(page_of(&harness).as_deref(), Some("viewer"));
    assert_eq!(title_of(&harness).as_deref(), Some("Viewer"));

    let html = harness.html();
    for words in [
        "Keep history",
        "Hide controls after",
        "Zoom step",
        "History",
        "Controls",
    ] {
        assert!(html.contains(words), "{words} is on the page");
    }
    assert_eq!(harness.count(".ds-toggle"), 1, "the history switch");
    assert_eq!(
        harness.attr(".ds-toggle", "aria-checked").as_deref(),
        Some("true"),
        "on, its default"
    );
    assert_eq!(
        harness.count(".ds-disclosure"),
        1,
        "the advanced zoom step is behind the page's one disclosure"
    );
}

#[test]
fn the_viewer_sits_after_the_fixed_pages_and_edits_anyviews_own_file() {
    let sandbox = Sandbox::new("viewer-writes");
    sandbox.install_schema(&anyview());
    let mut harness = sandbox.harness(opening("viewer"));
    let rows = harness.count(".ds-sidebar .ds-row");
    assert_eq!(
        harness.text_of(&format!(".ds-list-item:nth-child({rows}) .ds-row")),
        Some("Viewer".to_owned()),
        "the last of the pages the schemas make"
    );

    assert_eq!(sandbox.read(FILE), None, "looking writes nothing");
    click(&mut harness, ".ds-toggle");
    let text = sandbox.read(FILE).expect("anyview's file was written");
    assert!(text.contains("enabled = \"off\""), "{text}");

    let spec = sandbox
        .env(Opening::default(), ds::prelude::Scheme::Light)
        .catalogue
        .files
        .iter()
        .find(|f| f.file.relative().ends_with("anyview/settings.toml"))
        .cloned()
        .expect("anyview's file");
    let loaded = load(&sandbox.config_home(), &spec);
    assert_eq!(
        loaded
            .value
            .get(&KeyPath("viewer.history.enabled".to_owned())),
        Some(&toml::Value::String("off".to_owned()))
    );
}
