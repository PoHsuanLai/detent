//! How the window is dressed: the sidebar's groups and tiles, the person's card, the heroes of
//! the overview pages and the drill-in rows of the advanced keys.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, settle, sidebar_page};
use detent_model::PageSlug;
use detent_system::person::FakePerson;
use detent_ui::Opening;
use std::sync::Arc;

fn opening(page: &str) -> Opening {
    Opening {
        page: Some(PageSlug(page.to_owned())),
        query: String::new(),
    }
}

#[test]
fn every_page_row_has_a_tile_and_the_groups_are_set_apart_by_gaps() {
    let sandbox = Sandbox::new("look-sidebar");
    let harness = sandbox.harness(opening("dock"));
    let rows = harness.count(".ds-sidebar [data-page-row]");
    assert!(rows > 10, "{rows} pages");
    assert_eq!(
        harness.count(".ds-sidebar [data-page-row] .ds-icon-tile"),
        rows,
        "a tile on every row"
    );
    assert!(harness.count(".sidebar-gap") >= 4, "the groups have gaps");
}

#[test]
fn without_anyone_known_there_is_no_card() {
    let sandbox = Sandbox::new("look-no-person");
    let harness = sandbox.harness(opening("dock"));
    assert_eq!(harness.count("[data-person-card]"), 0);
}

#[test]
fn the_card_names_the_person_and_a_press_opens_accounts() {
    let sandbox =
        Sandbox::new("look-person").with_person(Arc::new(FakePerson::named("Ada Lovelace")));
    let mut harness = sandbox.harness(opening("dock"));
    settle(&mut harness, "the card", |h| {
        h.count("[data-person-card]") == 1
    });
    let card = harness.text_of("[data-person-card]").unwrap_or_default();
    assert!(card.contains("Ada Lovelace"), "{card}");
    click(&mut harness, "[data-person-card]");
    assert_eq!(page_of(&harness).as_deref(), Some("accounts"));
    assert_eq!(harness.count(&sidebar_page("accounts")), 1);
}

#[test]
fn only_the_overview_pages_open_with_a_hero() {
    let sandbox = Sandbox::new("look-heroes");
    for (page, heroes) in [
        ("privacy", 1),
        ("accounts", 1),
        ("intelligence", 1),
        ("dock", 0),
        ("notifications", 0),
    ] {
        let harness = sandbox.harness(opening(page));
        assert_eq!(harness.count("[data-hero]"), heroes, "{page}");
    }
}
