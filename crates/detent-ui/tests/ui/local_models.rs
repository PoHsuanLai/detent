//! The "Local models" group over a fake inferd (its running models on a private bus, its
//! picker module in memory): each model the assistant runs
//! itself with what it is doing, the memory it takes while ready and a Start or Stop; a Start that
//! waits shows progress until the assistant says the engine changed; a refusal shows its own
//! words under the row. And the rows that name an engine's program, set through the fake module.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::drive::{click, keys, scroll_into_view};
use crate::support::inferd::{Inferd, NOT_WHOLE};
use crate::support::running::{Fake, serve};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::ShortcutKey;
use ds_harness::{Harness, Input};
use ds_settings::live::Verdict;
use porter_dbus::{RunState, RunningModel};
use std::time::Duration;

const READY: &str = "[data-run=\"ready\"]";
const STOPPED: &str = "[data-run=\"stopped\"]";
const START_QWEN: &str = "[aria-label=\"Start qwen\"]";
const STOP_QWEN: &str = "[aria-label=\"Stop qwen\"]";
const NOTE: &str = "[data-note=\"model-run\"]";
const OPEN_PROGRAMS: &str = "[aria-label=\"Open Programs for local models\"]";
const VLLM: &str = "#program-engines-vllm-python";
const PROGRAM_NOTE: &str = "[data-note=\"program-refused\"]";

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fakes answer on real threads; a loaded machine running every test at once can take
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

struct World {
    fake: Fake,
    module: Inferd,
    harness: Harness,
    rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
}

fn qwen(state: RunState) -> RunningModel {
    RunningModel::new("local/qwen", state, "llama-server", 0)
}

/// The Intelligence page over inferd running `rows`, whose module lists the engine program rows.
fn world(name: &str, rows: Vec<RunningModel>) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon");
    let bus = PrivateBus::start(name);
    let module = Inferd::new(Verdict::Allow).on_slots().with_programs();
    let fake = rt.block_on(serve(&bus, rows));
    let harness = Sandbox::new(name)
        .on_bus(&bus.address)
        .with_modules(module.hosted())
        .harness(intelligence());
    World {
        fake,
        module,
        harness,
        rt,
        _bus: bus,
    }
}

fn row_text(harness: &Harness, selector: &str) -> String {
    harness.text_of(selector).unwrap_or_default()
}

#[test]
fn a_ready_model_shows_ready_the_memory_it_takes_and_stop_and_a_stopped_one_offers_start() {
    let mut w = world(
        "models-ready",
        vec![
            RunningModel::new("local/qwen", RunState::Ready, "llama-server", 14_540),
            RunningModel::new("local/holo", RunState::Stopped, "llama-server", 0),
        ],
    );
    settle(&mut w.harness, "the ready model", |h| h.count(READY) == 1);
    let ready = row_text(&w.harness, READY);
    assert!(
        ready.contains("Ready") && ready.contains("14.2 GB"),
        "{ready}"
    );
    assert_eq!(w.harness.count(STOP_QWEN), 1);
    let stopped = row_text(&w.harness, STOPPED);
    assert!(
        stopped.contains("Stopped") && !stopped.contains("GB"),
        "{stopped}"
    );
    assert_eq!(w.harness.count("[aria-label=\"Start holo\"]"), 1);
    assert!(w.fake.calls().is_empty(), "looking starts nothing");
}

#[test]
fn a_start_shows_progress_until_the_assistant_says_the_model_is_ready() {
    let mut w = world("models-start", vec![qwen(RunState::Stopped)]);
    settle(&mut w.harness, "the stopped model", |h| {
        h.count(START_QWEN) == 1
    });
    w.fake.hold_starts();
    scroll_into_view(&mut w.harness, START_QWEN);
    click(&mut w.harness, START_QWEN);
    settle(&mut w.harness, "the starting state", |h| {
        h.count("[data-run=\"starting\"]") == 1
    });
    assert_eq!(
        w.harness.attr(START_QWEN, "aria-busy").as_deref(),
        Some("true"),
        "the button works while the Start is in flight"
    );
    assert!(row_text(&w.harness, "[data-run=\"starting\"]").contains("Starting"));
    w.fake.finish();
    settle(&mut w.harness, "the ready state", |h| h.count(READY) == 1);
    assert!(row_text(&w.harness, READY).contains("14.2 GB"));
    assert_eq!(w.harness.count(STOP_QWEN), 1, "it can be stopped now");
    assert_eq!(w.fake.calls(), ["start local/qwen"]);
}

#[test]
fn a_refused_start_shows_the_assistants_words_under_the_row_and_changes_nothing() {
    let mut w = world("models-refused", vec![qwen(RunState::Stopped)]);
    settle(&mut w.harness, "the stopped model", |h| {
        h.count(START_QWEN) == 1
    });
    let words = "needs 14.2 GB of graphics memory; 6.1 GB is free";
    w.fake.refuse_next(words);
    scroll_into_view(&mut w.harness, START_QWEN);
    click(&mut w.harness, START_QWEN);
    settle(&mut w.harness, "the refusal", |h| h.count(NOTE) == 1);
    assert_eq!(row_text(&w.harness, NOTE), words);
    assert_eq!(w.harness.count(STOPPED), 1, "still stopped");
    assert_ne!(
        w.harness.attr(START_QWEN, "aria-busy").as_deref(),
        Some("true"),
        "the button can be pressed again"
    );
}

#[test]
fn stop_stops_the_model_and_a_failed_one_says_why() {
    let mut w = world(
        "models-stop",
        vec![RunningModel::new(
            "local/qwen",
            RunState::Ready,
            "llama-server",
            14_540,
        )],
    );
    settle(&mut w.harness, "the ready model", |h| {
        h.count(STOP_QWEN) == 1
    });
    scroll_into_view(&mut w.harness, STOP_QWEN);
    click(&mut w.harness, STOP_QWEN);
    settle(&mut w.harness, "the stopped state", |h| {
        h.count(STOPPED) == 1
    });
    assert_eq!(w.fake.calls(), ["stop local/qwen"]);
    let reason = "the engine did not become ready in time";
    w.rt.block_on(w.fake.fails("local/qwen", reason));
    settle(&mut w.harness, "the failure", |h| {
        h.count("[data-run=\"failed\"]") == 1
    });
    assert!(row_text(&w.harness, "[data-run=\"failed\"]").contains("Couldn\u{2019}t start"));
    assert_eq!(row_text(&w.harness, NOTE), reason);
}

#[test]
fn nothing_shows_while_the_assistant_is_not_running() {
    let mut harness = Sandbox::new("models-absent").harness(intelligence());
    for _ in 0..20 {
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(harness.count("[data-run]"), 0);
    assert_eq!(harness.count("[data-programs]"), 0);
}

/// Opens the sheet of programs.
fn programs_open(w: &mut World) {
    settle(&mut w.harness, "the programs row", |h| {
        h.count(OPEN_PROGRAMS) == 1
    });
    scroll_into_view(&mut w.harness, OPEN_PROGRAMS);
    click(&mut w.harness, OPEN_PROGRAMS);
    settle(&mut w.harness, "the sheet", |h| h.count(VLLM) == 1);
}

/// Types `text` into the field and presses Enter.
fn enter_path(w: &mut World, text: &str) {
    click(&mut w.harness, VLLM);
    keys(&mut w.harness, text);
    w.harness.send(Input::key(ShortcutKey::Enter));
    w.harness.advance(Duration::from_millis(400));
}

#[test]
fn an_engine_program_row_is_a_field_that_says_it_is_found_automatically_and_sets_through_the_module()
 {
    let mut w = world("programs-set", Vec::new());
    programs_open(&mut w);
    assert!(
        w.harness.html().contains("Found automatically"),
        "an empty field says what empty means"
    );
    enter_path(&mut w, "/usr/bin/python3");
    settle(&mut w.harness, "the set", |_| {
        !w.module.sets.lock().expect("sets").is_empty()
    });
    assert_eq!(
        *w.module.sets.lock().expect("sets"),
        [(
            "engines.vllm_python".to_owned(),
            "/usr/bin/python3".to_owned()
        )]
    );
    assert_eq!(w.harness.count(PROGRAM_NOTE), 0, "nothing was refused");
}

#[test]
fn a_path_the_module_refuses_shows_its_words_under_the_field_and_is_not_in_force() {
    let mut w = world("programs-refused", Vec::new());
    programs_open(&mut w);
    enter_path(&mut w, "python3");
    settle(&mut w.harness, "the refusal", |h| {
        h.count(PROGRAM_NOTE) == 1
    });
    assert_eq!(row_text(&w.harness, PROGRAM_NOTE), NOT_WHOLE);
    assert_eq!(
        w.module
            .programs
            .lock()
            .expect("programs")
            .get("engines.vllm_python")
            .map(String::as_str),
        Some(""),
        "the refused path is not in force"
    );
}
