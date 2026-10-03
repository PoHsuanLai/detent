//! The window told from outside while it runs: a page to open, files that changed under it.

mod support;

use detent_model::PageSlug;
use detent_ui::{Notice, Opening};
use ds_harness::{Driver, Query};
use std::time::Duration;
use support::Sandbox;
use support::drive::{click, page_of, search_for, title_of};

const SETTLE: Duration = Duration::from_millis(400);

fn slug(name: &str) -> PageSlug {
    PageSlug(name.to_owned())
}

#[test]
fn a_page_to_open_is_shown_and_clears_the_search() {
    let sandbox = Sandbox::new("notice-open-page");
    let (mut harness, outbox) = sandbox.harness_told(Opening::default());
    search_for(&mut harness, "dark mode");
    let narrowed = harness.count(".ds-sidebar .ds-row");
    assert_eq!(narrowed, 1, "the search narrowed the sidebar");

    outbox
        .send(Notice::OpenPage(slug("dock")))
        .expect("the window is listening");
    harness.advance(SETTLE);
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
    assert_eq!(title_of(&harness).as_deref(), Some("Dock"));
    assert!(
        harness.count(".ds-sidebar .ds-row") > narrowed,
        "no search is left narrowing the sidebar"
    );
}

#[test]
fn a_page_no_schema_makes_is_ignored() {
    let sandbox = Sandbox::new("notice-unknown-page");
    let (mut harness, outbox) = sandbox.harness_told(Opening::default());
    outbox
        .send(Notice::OpenPage(slug("nowhere")))
        .expect("the window is listening");
    harness.advance(SETTLE);
    assert_eq!(page_of(&harness).as_deref(), Some("appearance"));
}

#[test]
fn files_written_behind_the_window_show_after_a_reload() {
    let sandbox = Sandbox::new("notice-reload");
    let opening = Opening {
        page: Some(slug("dock")),
        query: String::new(),
    };
    let (mut harness, outbox) = sandbox.harness_told(opening);
    let switch = ".ds-toggle";
    assert_eq!(
        harness.attr(switch, "aria-checked").as_deref(),
        Some("true")
    );

    sandbox.write("sill/settings.toml", "[dock]\nmagnification = \"off\"\n");
    assert_eq!(
        harness.attr(switch, "aria-checked").as_deref(),
        Some("true"),
        "the window reads files once, until it is told"
    );
    outbox
        .send(Notice::Reload)
        .expect("the window is listening");
    harness.advance(SETTLE);
    assert_eq!(
        harness.attr(switch, "aria-checked").as_deref(),
        Some("false")
    );

    // The window still writes after a reload.
    click(&mut harness, switch);
    let text = sandbox.read("sill/settings.toml").expect("written");
    assert!(text.contains("magnification = \"on\""), "{text}");
}
