//! Automatic's list and Local's engine and model.

use super::*;

const POOL_ROW: &str = "#pool-local-qwen .ds-toggle";
const POOL_NOTE: &str = "Turn on at least one model.";

#[test]
fn automatic_opens_its_list_and_is_not_ready_until_a_model_is_in_it() {
    let mut world = world(
        "assistant-automatic",
        ASSISTANT_SETUP_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    assert_eq!(world.harness.count("[data-pool=\"editor\"]"), 0);
    open_and_count_marked(&mut world.harness, SETUP_BUTTON);
    pick(&mut world.harness, 1);
    settle(&mut world.harness, "Automatic", |h| {
        shown(h, SETUP_BUTTON).as_deref() == Some("Automatic")
    });
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        [(ROW.to_owned(), "auto".to_owned())]
    );
    assert_eq!(world.harness.count("[data-pool=\"editor\"]"), 1);
    // The note comes with the pool's first read, which can land a frame after the editor opens.
    settle(&mut world.harness, "an empty list says what to do", |h| {
        h.html().contains(POOL_NOTE)
    });
    assert_eq!(
        world.harness.attr(CONTINUE, "data-availability").as_deref(),
        Some("disabled"),
        "Automatic with nothing to choose from is not a choice yet"
    );

    scroll_into_view(&mut world.harness, POOL_ROW);
    click(&mut world.harness, POOL_ROW);
    settle(&mut world.harness, "the list", |h| {
        !h.html().contains(POOL_NOTE)
    });
    assert_eq!(
        *world.fake.pool_sets.lock().expect("pool sets"),
        [vec!["local/qwen".to_owned()]]
    );
    assert_ne!(
        world.harness.attr(CONTINUE, "data-availability").as_deref(),
        Some("disabled"),
        "a model to choose from makes it ready"
    );
}

#[test]
fn local_asks_how_it_runs_then_which_model_and_sets_only_the_model() {
    let mut world = world(
        "assistant-local",
        ASSISTANT_MODEL_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    open_and_count_marked(&mut world.harness, BUTTON);
    pick(&mut world.harness, 3);
    let model = "#picker-local-model-ai-model-text-balanced .ds-popup .ds-button";
    settle(&mut world.harness, "the questions", |h| h.count(model) == 1);
    assert_eq!(
        world.harness.count("#picker-engine-ai-model-text-balanced"),
        1,
        "how it runs is asked"
    );
    assert!(world.harness.html().contains("llama.cpp"));
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        Vec::<(String, String)>::new(),
        "picking Local sets nothing"
    );
    scroll_into_view(&mut world.harness, model);
    click(&mut world.harness, model);
    pick(&mut world.harness, 1);
    settle(&mut world.harness, "the set", |_| {
        world.fake.sets.lock().expect("sets").len() == 1
    });
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        [(ROW.to_owned(), "local/qwen".to_owned())]
    );
}
