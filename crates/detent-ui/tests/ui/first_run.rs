//! The first-run window over a fake inferd and a fake accountd on a private bus: AI stays on this
//! computer by default, a runtime found can be picked, Done writes through the Intelligence page's
//! own settings path and records that the first run happened.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held, Policy as AccountPolicy};
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

/// Everything a first run meets: inferd and accountd on one private bus, `local` a runtime on
/// this computer that is up and `lm-studio` one that is not.
struct World {
    sandbox: Sandbox,
    inferd: Inferd,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _daemons: Vec<zbus::Connection>,
}

fn world(name: &str) -> World {
    world_in(name, Scheme::Light)
}

fn world_in(name: &str, scheme: Scheme) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let inferd = Inferd::new(InferPolicy::Allow).on_slots();
    let accountd = Accountd::new(
        AccountPolicy::Allow,
        vec![
            Held::runtime("local", "Local", "ok"),
            Held::runtime("lm-studio", "LM Studio", "offline"),
        ],
    );
    let daemons = vec![
        rt.block_on(inferd.serve_on(&bus)),
        rt.block_on(accountd.serve_on(&bus)),
    ];
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    let mut harness = sandbox.first_run_harness_in(scheme);
    settle(&mut harness, "the window", |h| stage(h).is_some());
    World {
        sandbox,
        inferd,
        harness,
        _rt: rt,
        _bus: bus,
        _daemons: daemons,
    }
}

fn written(sandbox: &Sandbox) -> String {
    sandbox.read("quire/inferd.toml").unwrap_or_default()
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
        "no runtime picked, no model set"
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
fn a_runtime_that_is_up_can_be_picked_and_a_stopped_one_cannot() {
    let mut w = world("first-run-pick");
    settle(&mut w.harness, "the runtime menu", |h| h.count(MENU) == 1);
    click(&mut w.harness, MENU);
    assert_eq!(
        w.harness.attr(MENU, "aria-expanded").as_deref(),
        Some("true")
    );
    let labels = w.harness.html();
    for text in ["Automatic", "Local", "LM Studio"] {
        assert!(labels.contains(text), "{text}");
    }
    assert_eq!(
        w.harness
            .attr(".ds-menu-item:nth-child(3)", "aria-disabled")
            .as_deref(),
        Some("true"),
        "LM Studio is stopped"
    );
    let item = w
        .harness
        .centre(".ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("the menu is open");
    w.harness.send(Input::pointer_move(item));
    click(&mut w.harness, ".ds-menu-item:nth-child(2) .ds-menu-label");
    click(&mut w.harness, DONE);
    settle(&mut w.harness, "the save", |h| {
        stage(h).as_deref() == Some("done")
    });
    assert_eq!(
        *w.inferd.sets.lock().expect("sets"),
        [("ai.model.text.balanced".to_owned(), "local/qwen".to_owned())],
        "the runtime's first model, in the text slot's Balanced row"
    );
    assert_eq!(mark(&w.sandbox), FirstRunStanding::Done);
}

#[test]
fn without_a_runtime_found_there_is_no_menu_and_done_still_finishes() {
    let sandbox = Sandbox::new("first-run-no-runtime");
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
fn a_runtime_that_cannot_be_chosen_keeps_the_window_open_and_is_not_remembered() {
    // accountd lists a runtime but there is no inferd to choose its model at.
    let name = "first-run-no-inferd";
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let accountd = Accountd::new(
        AccountPolicy::Allow,
        vec![Held::runtime("local", "Local", "ok")],
    );
    let _daemon = rt.block_on(accountd.serve_on(&bus));
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    let mut harness = sandbox.first_run_harness();
    settle(&mut harness, "the runtime menu", |h| h.count(MENU) == 1);
    click(&mut harness, MENU);
    click(&mut harness, ".ds-menu-item:nth-child(2) .ds-menu-label");
    click(&mut harness, DONE);
    settle(&mut harness, "the refusal", |h| {
        h.html().contains("could not be chosen")
    });
    assert_eq!(
        stage(&harness).as_deref(),
        Some("asking"),
        "Done can be pressed again"
    );
    assert_eq!(mark(&sandbox), FirstRunStanding::Pending);
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
    assert_eq!(sandbox.read("quire/inferd.toml"), None);
}

#[test]
fn the_same_write_the_page_makes_leaves_every_other_key_of_the_file() {
    let sandbox = Sandbox::new("first-run-keeps-keys");
    sandbox.write("quire/inferd.toml", "# mine\n[ai]\nfuture = 3\n");
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
