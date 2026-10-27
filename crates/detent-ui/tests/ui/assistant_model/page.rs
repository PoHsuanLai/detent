//! The assistant's page: its place in the sidebar, nothing chosen, picking and accounts.

use super::*;

#[test]
fn the_page_follows_intelligence_in_the_sidebar_and_draws_only_the_assistants_row() {
    let mut world = world(
        "assistant-page",
        ASSISTANT_MODEL_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    // The fixtures' pages: Intelligence is the sixth, the page follows it.
    assert_eq!(
        world
            .harness
            .text_of(&sidebar_page("intelligence"))
            .as_deref(),
        Some("Intelligence")
    );
    assert_eq!(
        world
            .harness
            .text_of(&sidebar_page("assistant-model"))
            .as_deref(),
        Some("Assistant Model"),
        "the sidebar lists the page after Intelligence"
    );
    assert_eq!(
        world
            .harness
            .attr(&sidebar_page("assistant-model"), "aria-selected")
            .as_deref(),
        Some("true")
    );
    assert_eq!(
        world.harness.count(".assistant-model .ds-field-row"),
        1,
        "one row, not a card per slot and tier"
    );
    assert_eq!(world.harness.count(".picker-kind"), 0);
}

#[test]
fn a_model_nobody_chose_shows_a_prompt_and_marks_no_item_while_a_chosen_one_is_marked() {
    let mut empty = world(
        "assistant-unset",
        ASSISTANT_MODEL_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut empty);
    assert_eq!(
        shown(&empty.harness, BUTTON).as_deref(),
        Some("Choose\u{2026}"),
        "no model is claimed"
    );
    assert_eq!(
        open_and_count_marked(&mut empty.harness, BUTTON),
        0,
        "no row is selected"
    );
    let html = empty.harness.html();
    assert!(html.contains(">Off<"), "the empty word reads Off");
    for banned in ["Catalogue default", "Recommended", "recommended"] {
        assert!(!html.contains(banned), "{banned} is on the page:\n{html}");
    }

    let fake = Inferd::new(Policy::Allow).on_slots();
    fake.values
        .lock()
        .expect("values")
        .insert(ROW.to_owned(), "local/qwen".to_owned());
    let mut chosen = world("assistant-chosen", ASSISTANT_MODEL_SLUG, fake);
    ready(&mut chosen);
    assert_eq!(
        shown(&chosen.harness, BUTTON).as_deref(),
        Some("Local"),
        "a model that runs on this computer is the Local choice"
    );
    assert_eq!(
        open_and_count_marked(&mut chosen.harness, BUTTON),
        1,
        "the chosen one, and only it, is marked"
    );
}

#[test]
fn picking_a_model_sets_the_assistants_row_and_no_other() {
    let mut world = world(
        "assistant-pick",
        ASSISTANT_MODEL_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    open_and_count_marked(&mut world.harness, BUTTON);
    // Items: Off, Automatic, Local, the Anthropic header, then claude.
    pick(&mut world.harness, 5);
    settle(&mut world.harness, "the set", |h| {
        shown(h, BUTTON).as_deref() == Some("claude")
    });
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        [(ROW.to_owned(), "anthropic/claude".to_owned())]
    );
}

#[test]
fn a_model_that_needs_an_account_stays_pickable_and_offers_to_add_one_instead_of_setting() {
    let fake = Inferd::new(Policy::Allow).on_slots();
    fake.gated.lock().expect("gated").insert(
        "anthropic/claude".to_owned(),
        "Add an account to use".to_owned(),
    );
    let mut world = world("assistant-account", ASSISTANT_MODEL_SLUG, fake);
    ready(&mut world);
    open_and_count_marked(&mut world.harness, BUTTON);
    // Items: Off, Automatic, Local, the Anthropic header, then claude.
    let claude = ".ds-menu-item:nth-child(5)";
    assert_ne!(
        world.harness.attr(claude, "aria-disabled").as_deref(),
        Some("true"),
        "the choice is not greyed"
    );
    pick(&mut world.harness, 5);
    // No account service is on the bus, so the add sheet cannot open; the person is told calmly.
    settle(&mut world.harness, "the line", |h| {
        h.html()
            .contains("Could not add an account. Try again later.")
    });
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        Vec::<(String, String)>::new(),
        "nothing was set"
    );
    assert_eq!(
        shown(&world.harness, BUTTON).as_deref(),
        Some("Choose\u{2026}"),
        "and nothing is chosen"
    );
}
