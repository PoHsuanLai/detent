//! The page's sections and rows, rendered from the schemas.

use super::*;

#[test]
fn the_sections_follow_design_order_whatever_order_the_programs_were_read_in() {
    let sandbox = Sandbox::new("intelligence-order");
    // Memory and Computer use come from almanac's and cua's own schemas, Assistant from docket's;
    // sill's rows of the same Assistant section have no schema yet, so a stand-in names them.
    sandbox.add_schema(
        "sill-companion",
        &stand_in(
            "sill-companion",
            "sill-companion/companion.toml",
            "companion.summon",
            "Assistant",
        ),
    );
    let harness = sandbox.harness(intelligence());
    in_order(
        &harness.html(),
        &[
            "Assistant",
            "Models",
            "Model choice",
            "Memory",
            "Computer use",
            "Limits\u{2026}",
        ],
    );
}

#[test]
fn the_page_shows_inferds_and_dockets_file_rows_with_no_inferd_running() {
    let sandbox = Sandbox::new("intelligence-absent");
    let harness = sandbox.harness(intelligence());
    let html = harness.html();
    for row in ["Keep AI on this computer", "Mail", "Other apps"] {
        assert!(
            html.contains(row) || html.contains(&row.replace("&#39;", "'")),
            "{row}"
        );
    }
    assert_eq!(
        harness.attr(".picker", "data-picker").as_deref(),
        Some("unavailable"),
        "no bus: the picker is unavailable, the page is not"
    );
    assert!(html.contains("Unavailable"));
    assert_eq!(harness.count(".picker-kind"), 0);
}

/// Opens the picker's first fast row's menu.
fn open_first_menu(harness: &mut Harness) -> String {
    let button = ".picker-kind .ds-field-row:nth-child(1) .ds-popup .ds-button";
    scroll_into_view(harness, button);
    click(harness, button);
    assert_eq!(
        harness.attr(button, "aria-expanded").as_deref(),
        Some("true"),
        "the click opened the menu"
    );
    button.to_owned()
}

#[test]
fn rows_render_from_the_schemas_sections_and_labels_with_no_local_table() {
    let sandbox = Sandbox::new("intelligence-slots");
    let fake = Inferd::new(Policy::Allow).on_slots();
    let mut harness = sandbox.with_modules(fake.hosted()).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 3);
    assert_eq!(harness.count(".picker-kind .ds-field-row"), 5);
    assert_eq!(
        harness.count(".picker-area"),
        1,
        "one Models title over every slot"
    );
    in_order(
        &harness.html(),
        &[
            "Models",
            "Text",
            "Fast",
            "Balanced",
            "Demanding",
            "Speech to text",
            "Fast",
            "Computer use",
            "Balanced",
        ],
    );
}

#[test]
fn grouped_choices_render_under_headers_ungrouped_first_in_first_appearance_order() {
    let sandbox = Sandbox::new("intelligence-groups");
    let fake = Inferd::new(Policy::Allow).on_slots();
    let mut harness = sandbox.with_modules(fake.hosted()).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 3);
    open_first_menu(&mut harness);
    in_order(
        &harness.html(),
        &[
            "Off",
            "Automatic",
            "Local",
            "Anthropic",
            "claude",
            "Google",
            "gemini",
        ],
    );
}

#[test]
fn an_old_key_module_still_renders() {
    let sandbox = Sandbox::new("intelligence-old-keys");
    let fake = Inferd::new(Policy::Allow);
    let mut harness = sandbox.with_modules(fake.hosted()).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);
    assert_eq!(harness.count(".picker-area"), 1, "the section is the area");
    in_order(
        &harness.html(),
        &["Models", "Language", "Fast", "Computer use"],
    );
    open_first_menu(&mut harness);
    let html = harness.html();
    // The one "On this computer" is the assistant's own group; a picker group would be a second.
    assert_eq!(
        html.matches(">On this computer<").count(),
        1,
        "a module with no groups has no headers:\n{html}"
    );
}

#[test]
fn an_unavailable_choice_is_greyed_with_its_reason_and_cannot_be_set() {
    let sandbox = Sandbox::new("intelligence-unavailable");
    let fake = Inferd::new(Policy::Allow).on_slots();
    fake.gated.lock().expect("gated").insert(
        "google/gemini".to_owned(),
        "Needs a newer version".to_owned(),
    );
    let mut harness = sandbox.with_modules(fake.hosted()).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 3);
    open_first_menu(&mut harness);

    let html = harness.html();
    assert!(
        html.contains("Needs a newer version"),
        "the reason is under the choice:\n{html}"
    );
    // Items: "", auto, Local, the Anthropic header, claude, the Google header, gemini (disabled),
    // its reason.
    let gemini = ".ds-menu-item:nth-child(7)";
    assert_eq!(
        harness.attr(gemini, "aria-disabled").as_deref(),
        Some("true"),
        "gemini is greyed"
    );
    if let Some(centre) = harness.centre(&format!("{gemini} .ds-menu-label")) {
        harness.send(Input::pointer_move(centre));
        click(&mut harness, &format!("{gemini} .ds-menu-label"));
    }
    harness.advance(Duration::from_millis(300));
    assert_eq!(
        *fake.sets.lock().expect("sets"),
        Vec::<(String, String)>::new(),
        "a greyed choice sends no Set"
    );
}
