//! The looser memory settings over a fake memory on a private bus: saving a looser value on the
//! Intelligence page is the person's word and memory is told, a value that is not looser sends
//! nothing, a looser value typed into the file shows its note and Confirm, and a refusal is told
//! calmly.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::drive::{click, scroll_into_view};
use crate::support::memory::{Fake, serve};
use detent_model::{PageSlug, Trouble};
use detent_ui::Opening;
use ds_harness::{Driver, Harness, Query};
use std::time::Duration;

const PLAIN: &str =
    ".ds-segmented[aria-label=\"Memory files\"] .ds-segmented-segment:nth-of-type(2)";
const MANUAL: &str =
    ".ds-segmented[aria-label=\"Tidy memory\"] .ds-segmented-segment:nth-of-type(2)";
const NOTE: &str = "[data-consent=\"memory.files.at_rest\"]";
const CONFIRM: &str = "[aria-label=\"Confirm Memory files\"]";
const TYPED_BY_HAND: &str = "[memory.files]\nat_rest = \"plain\"\n";
const WAITS: &str = "Waits for you to confirm. Until then, memory keeps \u{201c}Private\u{201d}.";

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fake answers on real threads; a loaded machine running every test at once can take
    // several seconds, so the budget is generous (15 s) and costs nothing when it settles.
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

/// Lets whatever the window would still send reach the fake.
fn let_it_pass(harness: &mut Harness) {
    for _ in 0..20 {
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
}

struct World {
    sandbox: Sandbox,
    fake: Fake,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
}

/// The Intelligence page with memory on the bus reading the settings file, which holds `file`
/// to begin with when that is given. The page has read what waits before this returns.
fn world(name: &str, file: Option<&str>) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake");
    let bus = PrivateBus::start(name);
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    let path = sandbox.config_home().0.join("almanac/settings.toml");
    if let Some(text) = file {
        sandbox.write("almanac/settings.toml", text);
    }
    let fake = rt.block_on(serve(&bus, path));
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the first read", |_| fake.reads() >= 1);
    World {
        sandbox,
        fake,
        harness,
        _rt: rt,
        _bus: bus,
    }
}

#[test]
fn saving_a_looser_value_is_the_confirmation_and_memory_is_told_that_pair_alone() {
    let mut w = world("consent-save", None);
    assert!(w.fake.confirms().is_empty(), "looking confirms nothing");
    scroll_into_view(&mut w.harness, PLAIN);
    click(&mut w.harness, PLAIN);
    settle(&mut w.harness, "the confirmation", |_| {
        !w.fake.confirms().is_empty()
    });
    assert_eq!(w.fake.confirms(), ["memory.files.at_rest=plain"]);
    let file = w.sandbox.read("almanac/settings.toml").expect("the file");
    assert!(file.contains("at_rest = \"plain\""), "{file}");
    // The first read was the page's, the second looked at the save, the third came after the
    // confirmation.
    settle(&mut w.harness, "the list read again", |_| {
        w.fake.reads() >= 3
    });
    settle(&mut w.harness, "the note to go", |h| h.count(NOTE) == 0);
}

#[test]
fn a_value_that_is_not_looser_sends_nothing() {
    let mut w = world("consent-plain-value", None);
    let before = w.fake.reads();
    scroll_into_view(&mut w.harness, MANUAL);
    click(&mut w.harness, MANUAL);
    settle(&mut w.harness, "the save to be looked at", |_| {
        w.fake.reads() > before
    });
    let_it_pass(&mut w.harness);
    let file = w.sandbox.read("almanac/settings.toml").expect("the file");
    assert!(file.contains("when = \"manual\""), "{file}");
    assert!(w.fake.confirms().is_empty(), "{:?}", w.fake.confirms());
    assert_eq!(w.harness.count("[data-consent]"), 0);
}

#[test]
fn a_looser_value_typed_into_the_file_waits_with_its_note_until_confirm_is_pressed() {
    let mut w = world("consent-typed", Some(TYPED_BY_HAND));
    settle(&mut w.harness, "the note", |h| h.count(NOTE) == 1);
    assert!(w.harness.html().contains(WAITS), "{}", w.harness.html());
    assert_eq!(w.harness.count(CONFIRM), 1);
    assert!(w.fake.confirms().is_empty(), "looking confirms nothing");

    scroll_into_view(&mut w.harness, CONFIRM);
    click(&mut w.harness, CONFIRM);
    settle(&mut w.harness, "the note to go", |h| h.count(NOTE) == 0);
    assert_eq!(w.fake.confirms(), ["memory.files.at_rest=plain"]);
    let file = w.sandbox.read("almanac/settings.toml").expect("the file");
    assert_eq!(file, TYPED_BY_HAND, "confirming does not touch the file");
}

#[test]
fn a_refusal_is_told_in_a_calm_line_and_the_value_keeps_waiting() {
    let mut w = world("consent-refused", Some(TYPED_BY_HAND));
    settle(&mut w.harness, "the note", |h| h.count(NOTE) == 1);
    scroll_into_view(&mut w.harness, CONFIRM);
    for (refusal, trouble) in [
        ("Invalid", Trouble::Invalid),
        ("NotAllowed", Trouble::NotAllowed),
    ] {
        w.fake.refuse_next(refusal);
        click(&mut w.harness, CONFIRM);
        settle(&mut w.harness, refusal, |h| {
            h.html().contains(trouble.words())
        });
        assert_eq!(w.harness.count(NOTE), 1, "{refusal}: it still waits");
        assert!(
            w.fake.confirms().is_empty(),
            "{refusal}: nothing was confirmed"
        );
    }
    assert!(
        !w.harness.html().contains(Trouble::Invalid.words()),
        "the first line is replaced"
    );
}

#[test]
fn with_memory_not_running_the_page_has_no_note_and_saving_still_works() {
    let bus = PrivateBus::start("consent-absent");
    let sandbox = Sandbox::new("consent-absent").on_bus(&bus.address);
    sandbox.write("almanac/settings.toml", TYPED_BY_HAND);
    let mut harness = sandbox.harness(intelligence());
    let_it_pass(&mut harness);
    assert_eq!(harness.count("[data-consent]"), 0);
    scroll_into_view(&mut harness, MANUAL);
    click(&mut harness, MANUAL);
    let_it_pass(&mut harness);
    let file = sandbox.read("almanac/settings.toml").expect("the file");
    assert!(file.contains("when = \"manual\""), "{file}");
    assert_eq!(harness.count("[data-consent]"), 0);
}
