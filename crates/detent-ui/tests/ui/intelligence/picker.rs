//! The model picker: its groups, choosing a model and the footer.

use super::*;

#[test]
fn the_picker_lists_a_group_per_kind_and_a_row_per_tier_in_design_words() {
    let sandbox = Sandbox::new("intelligence-picker");
    let fake = Inferd::new(Policy::Allow);
    fake.values
        .lock()
        .expect("values")
        .insert("ai.model.llm.balanced".to_owned(), "local/qwen".to_owned());
    let mut harness = sandbox.with_modules(fake.hosted()).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);

    assert_eq!(
        harness.count(".picker-kind .ds-field-row"),
        4,
        "3 language tiers + 1 computer-use tier"
    );
    let html = harness.html();
    in_order(
        &html,
        &["Language", "Fast", "Balanced", "Demanding", "Computer use"],
    );
    assert!(
        !html.contains(">Best<"),
        "the Best slot is called Demanding"
    );
    assert_eq!(
        harness
            .text_of(".picker-kind .ds-field-row:nth-child(2) .ds-popup .ds-button-label")
            .as_deref()
            .map(str::trim),
        Some("qwen"),
        "the module's current value is the one shown"
    );
}

#[test]
fn choosing_a_model_shows_loading_it_until_inferd_answers_then_sets_it() {
    let sandbox = Sandbox::new("intelligence-choose");
    let fake = Inferd::new(Policy::Allow);
    *fake.held.lock().expect("held") = true;
    let mut harness = sandbox.with_modules(fake.hosted()).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);

    let fast = ".picker-kind .ds-field-row:nth-child(1)";
    let button = format!("{fast} .ds-popup .ds-button");
    scroll_into_view(&mut harness, &button);
    click(&mut harness, &button);
    assert_eq!(
        harness.attr(&button, "aria-expanded").as_deref(),
        Some("true"),
        "the click opened the menu"
    );
    // Items: Off, Automatic, Local, qwen, holo.
    let item = harness
        .centre(".ds-menu-item:nth-child(5) .ds-menu-label")
        .expect("the menu is open");
    harness.send(Input::pointer_move(item));
    click(&mut harness, ".ds-menu-item:nth-child(5) .ds-menu-label");
    settle(&mut harness, "Loading holo", |h| {
        h.html().contains("Loading holo")
    });
    assert_eq!(
        *fake.sets.lock().expect("sets"),
        [("ai.model.llm.fast".to_owned(), "local/holo".to_owned())],
        "exactly one Set, naming the row and the model"
    );
    assert_eq!(
        fake.values.lock().expect("values").get("ai.model.llm.fast"),
        None,
        "inferd has not put it in force yet"
    );

    fake.release.notify_one();
    settle(&mut harness, "the set to finish", |h| {
        !h.html().contains("Loading holo")
    });
    assert_eq!(
        fake.values
            .lock()
            .expect("values")
            .get("ai.model.llm.fast")
            .map(String::as_str),
        Some("local/holo")
    );
    assert_eq!(
        harness
            .text_of(&format!("{fast} .ds-button-label"))
            .as_deref()
            .map(str::trim),
        Some("holo")
    );
}

#[test]
fn the_footer_says_why_when_it_is_on_and_is_gone_when_it_is_off() {
    for (name, setting, shown) in [("on", None, true), ("off", Some("off"), false)] {
        let sandbox = Sandbox::new(&format!("intelligence-reason-{name}"));
        if let Some(word) = setting {
            sandbox.write(
                "inferd/inferd.toml",
                &format!("[ai.auto]\nshow_reason = \"{word}\"\n"),
            );
        }
        let fake = Inferd::new(Policy::Allow);
        let (events, feed) = ReasonFeed::channel();
        let mut harness = sandbox
            .with_reasons(feed)
            .with_modules(fake.hosted())
            .harness(intelligence());
        settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);
        assert_eq!(
            harness.count(".picker-reason"),
            0,
            "{name}: no footer before a reason"
        );

        events.send(InferEvent::Why(Why::Warm)).expect("sent");
        if shown {
            settle(&mut harness, "the reason", |h| {
                h.text_of(".picker-reason")
                    .is_some_and(|t| t.contains("Already running."))
            });
            events
                .send(InferEvent::Declined(Declined {
                    model: model("holo"),
                    because: DeclinedBecause::NotInstalled,
                }))
                .expect("sent");
            settle(&mut harness, "the refusal", |h| {
                h.text_of(".picker-reason").is_some_and(|t| {
                    t.contains("holo can't do this: it isn't on this computer yet.")
                })
            });
        } else {
            harness.advance(Duration::from_millis(200));
            assert_eq!(
                harness.count(".picker-reason"),
                0,
                "off: no footer whatever arrives"
            );
            assert!(!harness.html().contains("Already running."));
        }
    }
}
