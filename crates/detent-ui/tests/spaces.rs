//! The Spaces page's list over a fake registry and a fake memory on a private bus: Remove asks on
//! the sheet first, memory is told the choice before the registry is asked, and Cancel sends
//! nothing.

mod support;

use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::{Driver, Harness, Query};
use std::time::Duration;
use support::Sandbox;
use support::bus::PrivateBus;
use support::drive::click;
use support::spaces::{Fakes, serve};

const TRIP: &str = "[aria-label=\"Remove Trip\"]";
const SHEET: &str = ".space-sheet";
const REMOVE: &str = ".space-sheet .ds-button[data-answers=\"return\"]";
const CANCEL: &str = ".space-sheet .ds-button[data-answers=\"escape\"]";
const FLIP: &str = ".space-sheet .ds-button[data-variant=\"inline\"]";
const DELETE_HISTORY: &str = ".space-sheet [role=\"radio\"]:last-child";

fn spaces() -> Opening {
    Opening {
        page: Some(PageSlug("spaces".to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    for _ in 0..200 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

struct World {
    fakes: Fakes,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
}

fn world(name: &str, memory: bool) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fakes");
    let bus = PrivateBus::start(name);
    let fakes = rt.block_on(serve(
        &bus,
        &[("space-1", "Trip"), ("space-2", "Work")],
        memory,
    ));
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    let mut harness = sandbox.harness(spaces());
    settle(&mut harness, "the list", |h| h.count(TRIP) == 1);
    World {
        fakes,
        harness,
        _rt: rt,
        _bus: bus,
    }
}

fn log(w: &World) -> Vec<String> {
    w.fakes.log.lock().expect("log").clone()
}

fn open_sheet(w: &mut World) {
    click(&mut w.harness, TRIP);
    settle(&mut w.harness, "the sheet", |h| h.count(SHEET) == 1);
}

#[test]
fn the_sheet_opens_on_the_keeping_choices_and_says_its_words() {
    let mut w = world("spaces-sheet", true);
    assert!(log(&w).is_empty());
    open_sheet(&mut w);
    let html = w.harness.html();
    for text in [
        "Remove \u{201c}Trip\u{201d}?",
        "Its memories move to the apps that saved them.",
        "Keep its history",
        "Delete its history",
        "History is what happened in this Space, like files you saved and messages you sent.",
        "Delete its memories instead",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(
        w.harness
            .attr(".space-sheet [role=\"radio\"]", "aria-checked")
            .as_deref(),
        Some("true"),
        "the first choice is selected"
    );
    assert_eq!(
        w.harness.attr(REMOVE, "data-role").as_deref(),
        Some("normal"),
        "nothing is lost, so Remove is not destructive"
    );
}

#[test]
fn cancel_sends_nothing_and_the_space_stays() {
    let mut w = world("spaces-cancel", true);
    open_sheet(&mut w);
    click(&mut w.harness, CANCEL);
    settle(&mut w.harness, "the sheet to go", |h| h.count(SHEET) == 0);
    assert!(log(&w).is_empty());
    assert_eq!(w.harness.count(TRIP), 1);
}

#[test]
fn remove_tells_memory_first_and_then_the_registry_and_the_row_goes() {
    let mut w = world("spaces-remove", true);
    open_sheet(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(
        log(&w),
        [
            r#"memory space-1 {"memories":"move_to_apps","history":"keep"}"#.to_owned(),
            "registry remove space-1".to_owned(),
        ]
    );
    assert_eq!(w.harness.count("[aria-label=\"Remove Work\"]"), 1);
}

#[test]
fn the_link_flips_the_body_and_its_label_and_makes_remove_destructive() {
    let mut w = world("spaces-flip", true);
    open_sheet(&mut w);
    click(&mut w.harness, FLIP);
    settle(&mut w.harness, "the flipped body", |h| {
        h.html().contains("Its memories will be deleted.")
    });
    let html = w.harness.html();
    assert!(html.contains("Move its memories instead"));
    assert!(!html.contains("Delete its memories instead"));
    assert_eq!(
        w.harness.attr(REMOVE, "data-role").as_deref(),
        Some("destructive")
    );
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(
        log(&w)[0],
        r#"memory space-1 {"memories":"delete","history":"keep"}"#
    );
}

#[test]
fn deleting_the_history_alone_makes_remove_destructive_and_is_sent() {
    let mut w = world("spaces-history", true);
    open_sheet(&mut w);
    click(&mut w.harness, DELETE_HISTORY);
    settle(&mut w.harness, "the choice", |h| {
        h.attr(REMOVE, "data-role").as_deref() == Some("destructive")
    });
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(
        log(&w)[0],
        r#"memory space-1 {"memories":"move_to_apps","history":"delete"}"#
    );
}

#[test]
fn with_no_memory_there_is_nothing_to_settle_and_the_registry_is_still_asked() {
    let mut w = world("spaces-no-memory", false);
    open_sheet(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(log(&w), ["registry remove space-1".to_owned()]);
}
