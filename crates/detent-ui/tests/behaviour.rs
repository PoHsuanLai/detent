//! The window driven through the harness: pages, toggles, choices and search, on scratch
//! directories. Every write is read back from the scratch config home.

mod support;

use detent_files::load;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::Word;
use ds::prelude::{Point, Px};
use ds_harness::{Driver, Input, Query};
use std::time::Duration;
use support::Sandbox;
use support::drive::{click, page_of, search_for, sidebar_row, title_of};

fn opening(page: &str) -> Opening {
    Opening {
        page: Some(PageSlug(page.to_owned())),
        query: String::new(),
    }
}

#[test]
fn clicking_a_page_shows_it() {
    let sandbox = Sandbox::new("click-a-page");
    let mut harness = sandbox.harness(Opening::default());
    assert_eq!(page_of(&harness).as_deref(), Some("appearance"));
    assert_eq!(title_of(&harness).as_deref(), Some("Appearance"));

    click(&mut harness, &sidebar_row(2));
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
    assert_eq!(title_of(&harness).as_deref(), Some("Dock"));
    assert_eq!(
        harness.attr(&sidebar_row(2), "aria-selected").as_deref(),
        Some("true"),
        "the clicked row is the selected one"
    );
    assert_eq!(
        harness.attr(&sidebar_row(1), "aria-selected").as_deref(),
        Some("false")
    );
}

#[test]
fn toggling_a_switch_writes_the_file_and_the_value_round_trips() {
    let sandbox = Sandbox::new("toggle-writes");
    let mut harness = sandbox.harness(opening("dock"));
    assert_eq!(
        sandbox.read("sill/settings.toml"),
        None,
        "nothing is written by looking"
    );
    assert_eq!(
        harness.attr(".ds-toggle", "aria-checked").as_deref(),
        Some("true"),
        "magnification starts on, its default"
    );

    click(&mut harness, ".ds-toggle");
    assert_eq!(
        harness.attr(".ds-toggle", "aria-checked").as_deref(),
        Some("false"),
        "the switch turned off"
    );
    let text = sandbox
        .read("sill/settings.toml")
        .expect("the file was written");
    assert!(text.contains("magnification = \"off\""), "{text}");

    // Read back the way the owning program reads it, and by a second run of the app.
    let spec = harness_spec(&sandbox);
    let loaded = load(&sandbox.config_home(), &spec);
    let key = detent_model_key("dock.magnification");
    assert_eq!(
        loaded.value.get(&key),
        Some(&toml::Value::String("off".to_owned()))
    );
    let again = sandbox.harness(opening("dock"));
    assert_eq!(
        again.attr(".ds-toggle", "aria-checked").as_deref(),
        Some("false")
    );
}

#[test]
fn a_segmented_choice_writes_its_word() {
    let sandbox = Sandbox::new("segmented-writes");
    let mut harness = sandbox.harness(opening("appearance"));
    assert_eq!(sandbox.read("quire/appearance.toml"), None);
    click(&mut harness, ".ds-segmented-segment:nth-of-type(3)");

    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains("theme = \"dark\""), "{text}");
    assert_eq!(
        harness
            .attr(".ds-segmented-segment:nth-of-type(3)", "aria-checked")
            .as_deref(),
        Some("true"),
        "Dark is selected"
    );
}

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
fn a_hit_among_the_advanced_keys_opens_its_disclosure() {
    let sandbox = Sandbox::new("search-advanced");
    let mut harness = sandbox.harness(opening("dock"));
    let closed = ".ds-disclosure[*|data-shown=hidden] .ds-field-row";
    let hidden_rows = harness.count(closed);
    assert!(
        hidden_rows > 0,
        "before the search the advanced rows are behind closed disclosures"
    );
    search_for(&mut harness, "magnified size");
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
    let row = harness.text_of(".key-hit").expect("a highlighted row");
    assert!(row.contains("Magnified size"), "{row}");
    assert_eq!(
        harness.count(".ds-disclosure[*|data-shown=visible] .key-hit"),
        1,
        "the row sits in a disclosure the search opened"
    );
    assert_eq!(
        harness.count(".ds-disclosure[*|data-shown=hidden] .key-hit"),
        0,
        "and in none that is closed"
    );
}

#[test]
fn a_search_that_finds_nothing_says_so() {
    let sandbox = Sandbox::new("search-nothing");
    let mut harness = sandbox.harness(Opening::default());
    search_for(&mut harness, "zzzzqqqq");
    assert_eq!(harness.count(".ds-empty-state"), 1);
    assert_eq!(harness.count(".ds-sidebar .ds-row"), 0);
}

#[test]
fn a_kind_without_a_component_is_shown_read_only_with_a_note() {
    let sandbox = Sandbox::new("file-only");
    let mut harness = sandbox.harness(Opening::default());
    search_for(&mut harness, "control center modules");
    let row = harness.text_of(".key-hit").expect("a highlighted row");
    assert!(row.contains("Modules"), "{row}");
    assert!(row.contains("wifi, bluetooth"), "the value is shown: {row}");
    assert!(row.contains("Edit in file"), "{row}");
}

fn harness_spec(sandbox: &Sandbox) -> detent_model::FileSpec {
    sandbox
        .env(Opening::default(), ds::prelude::Scheme::Light)
        .catalogue
        .files
        .iter()
        .find(|f| {
            f.file.relative().ends_with("settings.toml") && f.file.relative().starts_with("sill")
        })
        .cloned()
        .expect("sill's file")
}

fn detent_model_key(path: &str) -> ds_settings::schema::KeyPath {
    ds_settings::schema::KeyPath(path.to_owned())
}

#[test]
fn choosing_an_accent_swatch_writes_that_accent() {
    let sandbox = Sandbox::new("swatch-writes");
    let mut harness = sandbox.harness(opening("appearance"));
    click(&mut harness, ".ds-radio-group-item:nth-of-type(3)");
    let want = ds::prelude::Accent::ALL[2].slug();
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains(&format!("accent = \"{want}\"")), "{text}");
    assert_eq!(
        harness
            .attr(".ds-radio-group-item:nth-of-type(3)", "aria-checked")
            .as_deref(),
        Some("true")
    );
}

#[test]
fn moving_a_slider_writes_a_number_in_its_range() {
    // The Size slider; the first slider in the document is inside a closed disclosure.
    const SIZE: &str = ".ds-slider[*|aria-label=Size]";
    let sandbox = Sandbox::new("slider-writes");
    let mut harness = sandbox.harness(opening("dock"));
    let track = harness.rect(SIZE).expect("a slider on the Dock page");
    let at = Point {
        x: Px(track.origin.x.0 + track.size.width.0 * 0.9),
        y: Px(track.origin.y.0 + track.size.height.0 / 2.0),
    };
    harness.send(Input::pointer_down(at));
    harness.advance(Duration::from_millis(200));
    harness.send(Input::pointer_up(at));
    harness.advance(Duration::from_millis(400));
    let text = sandbox
        .read("sill/settings.toml")
        .expect("the file was written");
    let table: toml::Table = text.parse().expect("valid toml");
    let size = table["dock"]["tile_size_px"]
        .as_integer()
        .expect("an integer");
    assert!(
        (74..=76).contains(&size),
        "a press 90% along 32..=80 stores about 75, not the default 48: {size}\n{text}"
    );
}

#[test]
fn a_write_that_fails_says_so_and_leaves_the_file_alone() {
    let sandbox = Sandbox::new("write-refused");
    sandbox.write("sill/settings.toml", "[dock\nmagnification = ");
    let mut harness = sandbox.harness(opening("dock"));
    click(&mut harness, ".ds-toggle");
    assert_eq!(harness.count(".ds-toast"), 1, "a toast reports the refusal");
    assert_eq!(
        sandbox.read("sill/settings.toml").as_deref(),
        Some("[dock\nmagnification = "),
        "what the person typed is kept"
    );
}
