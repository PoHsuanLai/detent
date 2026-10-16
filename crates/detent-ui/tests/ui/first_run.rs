//! The first-run window over a fake inferd on a private bus: AI stays on this computer by default,
//! the assistant's model is the picker's row with nothing chosen, Done writes through the
//! Intelligence page's own settings path and records that the first run happened.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::drive::click;
use crate::support::inferd::{Inferd, Policy as InferPolicy};
use detent_files::{FirstRunMark, FirstRunStanding};
use ds::prelude::Scheme;
use ds_harness::{Driver, Harness, Input, Query};
use std::time::Duration;

const LOCAL_ONLY: &str = ".first-run [aria-label=\"Keep AI on this computer\"] .ds-toggle";
const DONE: &str = ".first-run > .ds-button";
const MENU: &str = ".first-run .ds-popup .ds-button";
const POOL_NOTE: &str = "Pick at least one model for Automatic to choose from.";

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fake daemons answer on real threads; a loaded machine running every test at once can
    // take several seconds, so the budget is generous (15 s) and costs nothing when it settles.
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemons")
}

fn stage(harness: &Harness) -> Option<String> {
    harness.attr(".first-run", "data-first-run")
}

fn mark(sandbox: &Sandbox) -> FirstRunStanding {
    FirstRunMark::under(&sandbox.state_home()).standing()
}

/// Everything a first run meets: inferd on a private bus, serving the text slot.
struct World {
    sandbox: Sandbox,
    inferd: Inferd,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _daemon: zbus::Connection,
}

fn world(name: &str) -> World {
    world_in(name, Scheme::Light)
}

fn world_in(name: &str, scheme: Scheme) -> World {
    world_with(name, scheme, Inferd::new(InferPolicy::Allow).on_slots())
}

fn world_with(name: &str, scheme: Scheme, inferd: Inferd) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let daemon = rt.block_on(inferd.serve_on(&bus));
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    let mut harness = sandbox.first_run_harness_in(scheme);
    // The model row arrives after the first frame and moves Done down; a click aimed before it
    // lands misses. Wait for the whole window.
    settle(&mut harness, "the window", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("ready")
    });
    World {
        sandbox,
        inferd,
        harness,
        _rt: rt,
        _bus: bus,
        _daemon: daemon,
    }
}

fn written(sandbox: &Sandbox) -> String {
    sandbox.read("inferd/inferd.toml").unwrap_or_default()
}

#[test]
fn ai_stays_on_this_computer_by_default_and_done_writes_it_and_remembers() {
    let mut w = world("first-run-default");
    assert_eq!(
        w.harness.attr(LOCAL_ONLY, "aria-checked").as_deref(),
        Some("true")
    );
    assert_eq!(mark(&w.sandbox), FirstRunStanding::Pending);
    click(&mut w.harness, DONE);
    settle(&mut w.harness, "the save", |h| {
        stage(h).as_deref() == Some("done")
    });
    assert!(
        written(&w.sandbox).contains("local_only = \"on\""),
        "{}",
        written(&w.sandbox)
    );
    assert_eq!(mark(&w.sandbox), FirstRunStanding::Done);
    assert_eq!(
        *w.inferd.sets.lock().expect("sets"),
        [],
        "nothing chosen, no model set"
    );
}

/// Opens the model menu and returns how many items are marked chosen.
fn open_and_count_marked(w: &mut World) -> usize {
    click(&mut w.harness, MENU);
    assert_eq!(
        w.harness.attr(MENU, "aria-expanded").as_deref(),
        Some("true")
    );
    w.harness.count(".ds-menu-item[aria-checked=\"true\"]")
}

/// Clicks the menu item `nth` (from 1), the way a person points at it first.
fn pick(w: &mut World, nth: usize) {
    let label = format!(".ds-menu-item:nth-child({nth}) .ds-menu-label");
    let at = w.harness.centre(&label).expect("the menu is open");
    w.harness.send(Input::pointer_move(at));
    click(&mut w.harness, &label);
}

#[test]
fn the_model_is_nothing_chosen_until_the_person_chooses() {
    let mut w = world("first-run-nothing");
    assert_eq!(
        w.harness
            .text_of(&format!("{MENU} .ds-button-label"))
            .as_deref()
            .map(str::trim),
        Some("Choose\u{2026}")
    );
    assert_eq!(open_and_count_marked(&mut w), 0, "no item is marked");
    assert!(
        !w.harness.html().contains(POOL_NOTE),
        "Automatic is not opened"
    );
}

#[test]
fn automatic_opens_its_list_and_local_asks_how_it_runs_then_which_model() {
    let mut w = world("first-run-modes");
    open_and_count_marked(&mut w);
    // Items: None, Automatic, Local, the Anthropic header, claude, the Google header, gemini.
    pick(&mut w, 2);
    settle(&mut w.harness, "Automatic", |h| {
        h.html().contains(POOL_NOTE)
    });
    assert_eq!(
        *w.inferd.sets.lock().expect("sets"),
        [("ai.model.text.balanced".to_owned(), "auto".to_owned())]
    );
    assert_eq!(w.harness.count("[data-pool=\"editor\"]"), 1);

    click(&mut w.harness, MENU);
    pick(&mut w, 3);
    let model = "#picker-local-model-ai-model-text-balanced .ds-popup .ds-button";
    settle(&mut w.harness, "the questions", |h| h.count(model) == 1);
    assert!(w.harness.html().contains("llama.cpp"));
    assert_eq!(
        w.inferd.sets.lock().expect("sets").len(),
        1,
        "picking Local sets nothing"
    );
    click(&mut w.harness, model);
    pick(&mut w, 1);
    settle(&mut w.harness, "the set", |_| {
        w.inferd.sets.lock().expect("sets").len() == 2
    });
    assert_eq!(
        w.inferd.sets.lock().expect("sets")[1],
        ("ai.model.text.balanced".to_owned(), "local/qwen".to_owned())
    );
}

#[test]
fn the_modules_note_shows_under_the_row_and_under_the_list_once() {
    const NOTE: &str = "Running models on this computer needs llama.cpp, which is not installed.";
    let note = |w: &World, kind: &str| w.harness.count(&format!("[data-note=\"{kind}\"]"));
    let inferd = Inferd::new(InferPolicy::Allow).on_slots().noting(NOTE);
    let mut w = world_with("first-run-note", Scheme::Light, inferd);
    assert_eq!((note(&w, "row"), note(&w, "pool")), (1, 0));
    assert!(w.harness.html().contains(NOTE));
    open_and_count_marked(&mut w);
    pick(&mut w, 2);
    settle(&mut w.harness, "Automatic", |h| {
        h.html().contains(POOL_NOTE)
    });
    assert_eq!(
        (note(&w, "row"), note(&w, "pool")),
        (0, 1),
        "Automatic's list says the note, the row does not repeat it"
    );
}

#[test]
fn switching_it_off_writes_off() {
    let mut w = world("first-run-off");
    click(&mut w.harness, LOCAL_ONLY);
    assert_eq!(
        w.harness.attr(LOCAL_ONLY, "aria-checked").as_deref(),
        Some("false")
    );
    click(&mut w.harness, DONE);
    settle(&mut w.harness, "the save", |h| {
        stage(h).as_deref() == Some("done")
    });
    assert!(written(&w.sandbox).contains("local_only = \"off\""));
}

#[test]
fn without_a_bus_there_is_no_model_menu_and_done_still_finishes() {
    let sandbox = Sandbox::new("first-run-no-bus");
    let mut harness = sandbox.first_run_harness();
    settle(&mut harness, "the window", |h| stage(h).is_some());
    assert_eq!(harness.count(MENU), 0);
    click(&mut harness, DONE);
    settle(&mut harness, "the save", |h| {
        stage(h).as_deref() == Some("done")
    });
    assert!(written(&sandbox).contains("local_only = \"on\""));
    assert_eq!(mark(&sandbox), FirstRunStanding::Done);
}

#[test]
fn without_inferds_schema_nothing_is_written_and_nothing_is_remembered() {
    let sandbox = Sandbox::new("first-run-no-schema");
    sandbox.remove_schema("inferd");
    let mut harness = sandbox.first_run_harness();
    settle(&mut harness, "the window", |h| stage(h).is_some());
    click(&mut harness, DONE);
    settle(&mut harness, "the refusal", |h| {
        h.html().contains("The model settings are not installed")
    });
    assert_eq!(mark(&sandbox), FirstRunStanding::Pending);
    assert_eq!(sandbox.read("inferd/inferd.toml"), None);
}

#[test]
fn the_same_write_the_page_makes_leaves_every_other_key_of_the_file() {
    let sandbox = Sandbox::new("first-run-keeps-keys");
    sandbox.write("inferd/inferd.toml", "# mine\n[ai]\nfuture = 3\n");
    let mut harness = sandbox.first_run_harness();
    settle(&mut harness, "the window", |h| stage(h).is_some());
    click(&mut harness, DONE);
    settle(&mut harness, "the save", |h| {
        stage(h).as_deref() == Some("done")
    });
    let text = written(&sandbox);
    assert!(
        text.contains("# mine") && text.contains("future = 3"),
        "{text}"
    );
    assert!(text.contains("local_only = \"on\""), "{text}");
}
