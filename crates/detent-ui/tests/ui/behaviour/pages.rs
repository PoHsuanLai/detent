//! The pages' own shapes: read-only kinds, sheet rows, gesture actions and saving.

use super::*;

#[test]
fn a_kind_without_a_component_is_shown_read_only_with_a_note() {
    let sandbox = Sandbox::new("file-only");
    let mut harness = sandbox.harness(Opening::default());
    search_for(&mut harness, "control center modules");
    let row = harness.text_of(".key-hit").expect("a highlighted row");
    assert!(row.contains("Control Center"), "the section's row: {row}");
    click(&mut harness, ".key-hit");
    harness.advance(Duration::from_millis(800));
    let sheet = harness
        .text_of(".advanced-page")
        .expect("the section's page");
    assert!(sheet.contains("Modules"), "{sheet}");
    assert!(
        sheet.contains("wifi, bluetooth"),
        "the value is shown: {sheet}"
    );
    assert!(sheet.contains("Edit in file"), "{sheet}");
}

#[test]
fn a_page_ends_with_one_group_of_sheet_rows_one_per_advanced_section() {
    let sandbox = Sandbox::new("one-advanced");
    // The advanced sections each page has: Menu bar, Clipboard, Search, Control Center and
    // Display; Dock; Momentum and Scroll bars.
    for (page, rows) in [("appearance", 5), ("dock", 1), ("mouse_and_gestures", 2)] {
        let harness = sandbox.harness(opening(page));
        assert_eq!(harness.count(".ds-disclosure"), 0, "{page}: no disclosure");
        assert_eq!(
            harness.count(".advanced .ds-field-group"),
            1,
            "{page}: one group"
        );
        assert_eq!(
            harness.count(".advanced [data-advanced-row]"),
            rows,
            "{page}: a row for each advanced section"
        );
        assert_eq!(harness.count(".ds-sheet"), 0, "{page}: sheets start closed");
        let html = harness.html();
        let last_section = html.rfind("class=\"section\"").expect("a section");
        let advanced = html.find("class=\"advanced\"").expect("the group");
        assert!(last_section < advanced, "{page}: the group is last");
    }
}

const CUSTOM_ACTION: &str = "two_finger_swipe_left = { kind = \"keys\", v = \"alt+left\" }";

#[test]
fn a_custom_gesture_action_is_read_only_and_survives_a_save() {
    let sandbox = Sandbox::new("custom-action");
    let before =
        format!("# mine\n[gestures.gesture_action_map]\n{CUSTOM_ACTION}\n[feel]\nfuture_key = 3\n");
    sandbox.write("palmrest/gestures.toml", &before);
    let mut harness = sandbox.harness(opening("mouse_and_gestures"));
    let row = harness
        .text_of(".key-file-only")
        .expect("the custom action row is read-only");
    assert!(row.contains("Edit in file"), "{row}");
    assert_eq!(harness.count(".key-file-only"), 1, "only the custom action");

    // Scroll speed is palmrest's own key: a press on its slider writes into the same file.
    click(&mut harness, ".ds-slider[*|aria-label=\"Scroll speed\"]");
    let after = sandbox.read("palmrest/gestures.toml").expect("written");
    let table: toml::Table = after.parse().expect("valid toml");
    let speed = table["scroll"]["speed"].as_integer().expect("an integer");
    assert!(
        (30..=33).contains(&speed),
        "a press in the middle of 0..=63 stores about 31: {speed}\n{after}"
    );
    for kept in ["# mine", CUSTOM_ACTION, "future_key = 3"] {
        assert!(after.contains(kept), "{kept:?} survives the save: {after}");
    }
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
    click(&mut harness, &theme_choice(3));
    let text = sandbox.read("quire/appearance.toml").expect("written");
    assert!(text.starts_with("version = 1\n"), "{text}");
    assert!(text.contains("old_key = 1"), "{text}");
}
