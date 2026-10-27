//! Searching the sidebar and the rows.

use super::*;

#[test]
fn searching_highlights_the_matching_row_and_shows_its_page() {
    let sandbox = Sandbox::new("search-highlights");
    let mut harness = sandbox.harness(opening("dock"));
    assert_eq!(harness.count(".key-hit"), 0, "no search, no highlight");

    search_for(&mut harness, "dark mode");
    assert_eq!(
        page_of(&harness).as_deref(),
        Some("appearance"),
        "the page with the hit"
    );
    assert_eq!(harness.count(".key-hit"), 1);
    let row = harness.text_of(".key-hit").expect("a highlighted row");
    assert!(row.contains("Theme"), "{row}");
    assert_eq!(
        harness.count(".ds-sidebar .ds-row"),
        1,
        "only the page with a hit is listed"
    );
}

#[test]
fn typing_the_start_of_a_page_name_keeps_that_page_and_shows_it() {
    let sandbox = Sandbox::new("search-page-name");
    let mut harness = sandbox.harness(opening("dock"));
    search_for(&mut harness, "acc");
    assert_eq!(
        page_of(&harness).as_deref(),
        Some("accounts"),
        "the page the name starts"
    );
    assert_eq!(title_of(&harness).as_deref(), Some("Accounts"));
    let listed = harness.html();
    assert!(
        listed.contains("data-page=\"accounts\""),
        "the Accounts page is not dropped"
    );
}

#[test]
fn a_hit_among_the_advanced_keys_marks_the_row_of_its_section() {
    let sandbox = Sandbox::new("search-advanced");
    let mut harness = sandbox.harness(opening("dock"));
    assert_eq!(
        harness.count(".advanced .key-hit"),
        0,
        "before the search no advanced row is marked"
    );
    search_for(&mut harness, "magnified size");
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
    assert_eq!(
        harness.count(".advanced .key-hit"),
        1,
        "the section holding the key is marked, and no sheet is opened for it"
    );
    assert_eq!(harness.count(".ds-sheet"), 0);
}

#[test]
fn a_search_that_finds_nothing_says_so() {
    let sandbox = Sandbox::new("search-nothing");
    let mut harness = sandbox.harness(Opening::default());
    search_for(&mut harness, "zzzzqqqq");
    assert_eq!(harness.count(".ds-empty-state"), 1);
    assert_eq!(harness.count(".ds-sidebar .ds-row"), 0);
}
