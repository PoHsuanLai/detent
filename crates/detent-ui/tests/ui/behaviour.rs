//! The window driven through the harness: pages, toggles, choices and search, on scratch
//! directories. Every write is read back from the scratch config home.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, search_for, sidebar_row, title_of};
use detent_files::load;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::Word;
use ds::prelude::{Point, Px};
use ds_harness::Input;
use std::time::Duration;

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

    click(&mut harness, &sidebar_row(9));
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
    assert_eq!(title_of(&harness).as_deref(), Some("Desktop & Dock"));
    assert_eq!(
        harness.attr(&sidebar_row(9), "aria-selected").as_deref(),
        Some("true"),
        "the clicked row is the selected one"
    );
    assert_eq!(
        harness.attr(&sidebar_row(4), "aria-selected").as_deref(),
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

#[test]
fn a_kind_without_a_component_is_shown_read_only_with_a_note() {
    let sandbox = Sandbox::new("file-only");
    let mut harness = sandbox.harness(Opening::default());
    search_for(&mut harness, "control center modules");
    let row = harness.text_of(".key-hit").expect("a highlighted row");
    assert!(row.contains("Control center"), "the section's row: {row}");
    click(&mut harness, ".key-hit .ds-button");
    let sheet = harness.text_of(".ds-sheet").expect("the section's sheet");
    assert!(sheet.contains("Modules"), "{sheet}");
    assert!(
        sheet.contains("wifi, bluetooth"),
        "the value is shown: {sheet}"
    );
    assert!(sheet.contains("Edit in file"), "{sheet}");
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

#[test]
fn reduce_motion_is_a_switch_that_is_on_for_reduced() {
    let sandbox = Sandbox::new("reduce-motion");
    let mut harness = sandbox.harness(opening("appearance"));
    let switch = ".section .ds-toggle[*|aria-label=\"Reduce motion\"]";
    assert_eq!(
        harness.count(switch),
        1,
        "a two-word key that is on and off is one switch"
    );
    assert_eq!(
        harness.attr(switch, "aria-checked").as_deref(),
        Some("false"),
        "Standard is off"
    );
    click(&mut harness, switch);
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains("motion_level = \"reduced\""), "{text}");
    assert_eq!(
        harness.attr(switch, "aria-checked").as_deref(),
        Some("true")
    );
}

#[test]
fn a_two_word_choice_that_is_not_on_and_off_is_two_segments() {
    let sandbox = Sandbox::new("two-segments");
    let mut harness = sandbox.harness(opening("appearance"));
    let typeface = ".ds-segmented[*|aria-label=Typeface]";
    assert_eq!(
        harness.count(&format!("{typeface} .ds-segmented-segment")),
        2,
        "System and Editorial"
    );
    click(
        &mut harness,
        &format!("{typeface} .ds-segmented-segment:nth-of-type(2)"),
    );
    let text = sandbox
        .read("quire/appearance.toml")
        .expect("the file was written");
    assert!(text.contains("typeface = \"editorial\""), "{text}");
}

#[test]
fn a_page_ends_with_one_group_of_sheet_rows_one_per_advanced_section() {
    let sandbox = Sandbox::new("one-advanced");
    for page in ["appearance", "dock", "mouse_and_gestures"] {
        let harness = sandbox.harness(opening(page));
        assert_eq!(harness.count(".ds-disclosure"), 0, "{page}: no disclosure");
        assert_eq!(
            harness.count(".advanced .ds-field-group"),
            1,
            "{page}: one group"
        );
        assert!(
            harness.count(".advanced .ds-field-row") > 1,
            "{page}: a row for each advanced section"
        );
        assert_eq!(harness.count(".ds-sheet"), 0, "{page}: sheets start closed");
        let html = harness.html();
        let last_section = html.rfind("class=\"section\"").expect("a section");
        let advanced = html.find("class=\"advanced\"").expect("the group");
        assert!(last_section < advanced, "{page}: the group is last");
        assert!(html.contains('\u{2026}'), "{page}: rows end in an ellipsis");
    }
}

const CUSTOM_ACTION: &str = "two_finger_swipe_left = { kind = \"keys\", v = \"alt+left\" }";

#[test]
fn a_custom_gesture_action_is_read_only_and_survives_a_save() {
    let sandbox = Sandbox::new("custom-action");
    let before = format!(
        "# mine\nrejection = \"on\"\n[gestures.gesture_action_map]\n{CUSTOM_ACTION}\n[feel]\nfuture_key = 3\n"
    );
    sandbox.write("palmrest/gestures.toml", &before);
    let mut harness = sandbox.harness(opening("mouse_and_gestures"));
    let row = harness
        .text_of(".key-file-only")
        .expect("the custom action row is read-only");
    assert!(row.contains("Edit in file"), "{row}");
    assert_eq!(harness.count(".key-file-only"), 1, "only the custom action");

    click(&mut harness, ".section .ds-toggle");
    let after = sandbox.read("palmrest/gestures.toml").expect("written");
    assert_eq!(
        after,
        before.replace("\"on\"", "\"off\""),
        "one value changed"
    );
}

#[test]
fn a_gesture_action_menu_reads_in_the_schemas_words() {
    let sandbox = Sandbox::new("action-labels");
    let harness = sandbox.harness(opening("mouse_and_gestures"));
    let html = harness.html();
    for label in [
        "Previous workspace",
        "Next workspace",
        "Workspace overview",
        "Smart zoom",
    ] {
        assert!(html.contains(label), "{label}");
    }
    assert!(!html.contains("Workspace prev<"), "no code-like label");
}

#[test]
fn saving_never_removes_a_key_from_sill_or_quire_files() {
    let sandbox = Sandbox::new("keep-unknown");
    let sill = "[dock]\npuppy = true\nposition = \"sideways\"\n[mystery]\nx = 1\n";
    let quire = "version = 1\n[appearance]\nold_key = 1\n";
    sandbox.write("sill/settings.toml", sill);
    sandbox.write("quire/appearance.toml", quire);

    let mut harness = sandbox.harness(opening("dock"));
    click(&mut harness, ".ds-toggle");
    let text = sandbox.read("sill/settings.toml").expect("written");
    assert!(
        text.contains("puppy = true") && text.contains("[mystery]\nx = 1"),
        "{text}"
    );
    assert!(
        text.contains("position = \"sideways\""),
        "an unknown word stays: {text}"
    );

    let mut harness = sandbox.harness(opening("appearance"));
    click(&mut harness, ".ds-segmented-segment:nth-of-type(3)");
    let text = sandbox.read("quire/appearance.toml").expect("written");
    assert!(text.starts_with("version = 1\n"), "{text}");
    assert!(text.contains("old_key = 1"), "{text}");
}
