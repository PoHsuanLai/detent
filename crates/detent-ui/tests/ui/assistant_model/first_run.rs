//! The first-run step: no sidebar, Continue, Skip and the step opened by a notice.

use super::*;

#[test]
fn the_first_run_step_has_no_sidebar_and_continue_waits_for_a_choice() {
    let mut world = world(
        "assistant-setup",
        ASSISTANT_SETUP_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    assert_eq!(world.harness.count(".assistant-setup"), 1);
    assert_eq!(
        world.harness.count(".ds-sidebar"),
        0,
        "a dialogue, not the window"
    );
    let html = world.harness.html();
    for text in ["Choose a Model for Your Assistant", "Skip", "Continue"] {
        assert!(html.contains(text), "{text} is on the step");
    }
    assert_eq!(
        shown(&world.harness, SETUP_BUTTON).as_deref(),
        Some("Choose\u{2026}")
    );
    assert_eq!(
        world.harness.attr(CONTINUE, "data-availability").as_deref(),
        Some("disabled"),
        "nothing is chosen, so Continue waits"
    );
    assert_ne!(
        world.harness.attr(SKIP, "data-availability").as_deref(),
        Some("disabled"),
        "Skip never waits"
    );

    open_and_count_marked(&mut world.harness, SETUP_BUTTON);
    pick(&mut world.harness, 4);
    settle(&mut world.harness, "Continue", |h| {
        h.attr(CONTINUE, "data-availability").as_deref() != Some("disabled")
    });
}

#[test]
fn skip_writes_nothing() {
    let mut world = world(
        "assistant-skip",
        ASSISTANT_SETUP_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    click(&mut world.harness, SKIP);
    world.harness.advance(Duration::from_millis(300));
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        Vec::<(String, String)>::new(),
        "the model stays unset"
    );
}

#[test]
fn a_page_to_open_can_be_the_first_run_step() {
    let sandbox = Sandbox::new("assistant-notice");
    let (mut harness, outbox) = sandbox.harness_told(Opening::default());
    assert_eq!(harness.count(".assistant-setup"), 0);
    outbox
        .send(Notice::OpenPage(PageSlug(ASSISTANT_SETUP_SLUG.to_owned())))
        .expect("the window is listening");
    harness.advance(Duration::from_millis(400));
    assert_eq!(harness.count(".assistant-setup"), 1);
    assert_eq!(harness.count(".ds-sidebar"), 0);
    outbox
        .send(Notice::OpenPage(PageSlug(ASSISTANT_MODEL_SLUG.to_owned())))
        .expect("the window is listening");
    harness.advance(Duration::from_millis(400));
    assert_eq!(harness.count(".assistant-setup"), 0);
    assert_eq!(
        harness.attr(".scroll", "data-page").as_deref(),
        Some(ASSISTANT_MODEL_SLUG),
        "the window is back, on the page"
    );
}
