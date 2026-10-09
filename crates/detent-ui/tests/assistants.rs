//! The Intelligence page's assistants: where the built-in assistant may run (this computer, the
//! person's computers, signed-in cloud accounts), written to docket's settings file, and the
//! assistants that sign themselves in.

mod support;

use detent_model::PageSlug;
use detent_ui::Opening;
use docket_settings::{CloudAllowed, Toggle, read_places};
use ds_harness::{Driver, Harness, Query};
use std::time::Duration;
use support::Sandbox;
use support::accountd::{Accountd, Held, Policy};
use support::bus::PrivateBus;
use support::drive::click;

const CLOUD: &str = "[aria-label=\"me@acme.example\"] .ds-toggle";
const LAB: &str = "[aria-label=\"lab server\"] .ds-toggle";
const SETTINGS: &str = "docket/settings.toml";

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
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

/// The page over a fake accountd holding `held`, in a world whose docket file holds `file`.
struct World {
    sandbox: Sandbox,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _daemon: zbus::Connection,
}

fn world(name: &str, held: Vec<Held>, file: Option<&str>) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon");
    let bus = PrivateBus::start(name);
    let fake = Accountd::new(Policy::Allow, held);
    let daemon = rt.block_on(fake.serve_on(&bus));
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    if let Some(text) = file {
        sandbox.write(SETTINGS, text);
    }
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the places", |h| {
        h.count("[data-assistants=\"places\"]") == 1
    });
    World {
        sandbox,
        harness,
        _rt: rt,
        _bus: bus,
        _daemon: daemon,
    }
}

fn acme() -> Held {
    Held::new("acme", "me@acme.example", "ok", &["llm"])
        .in_group("intelligence")
        .of_provider("acme", "Acme")
}

fn written(sandbox: &Sandbox) -> Option<docket_settings::Places> {
    sandbox.read(SETTINGS).map(|text| read_places(&text).places)
}

fn the_cloud_account_is_listed(w: &mut World) {
    settle(&mut w.harness, "the account", |h| h.count(CLOUD) == 1);
}

#[test]
fn this_computer_is_always_on_with_no_switch_and_a_cloud_account_starts_off() {
    let mut w = world("assistants-groups", vec![acme()], None);
    the_cloud_account_is_listed(&mut w);
    let html = w.harness.html();
    for text in [
        "On this computer",
        "Your computers",
        "Cloud accounts",
        "Always on",
        "Acme",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(
        w.harness.count("[aria-label=\"This computer\"] .ds-toggle"),
        0
    );
    assert_eq!(
        w.harness.attr(CLOUD, "aria-checked").as_deref(),
        Some("false")
    );
    assert_eq!(written(&w.sandbox), None, "reading writes nothing");
}

#[test]
fn turning_a_cloud_account_on_asks_in_plain_words_and_yes_writes_it() {
    let mut w = world("assistants-confirm", vec![acme()], None);
    the_cloud_account_is_listed(&mut w);
    click(&mut w.harness, CLOUD);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Turn on me@acme.example?")
    });
    let html = w.harness.html().replace("&#39;", "'");
    assert!(
        html.contains("Requests the assistant can't do on your computers may be sent to Acme."),
        "{html}"
    );
    assert_eq!(written(&w.sandbox), None, "the press alone writes nothing");
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-answers=\"return\"]",
    );
    settle(&mut w.harness, "the write", |_| {
        written(&w.sandbox).is_some()
    });
    let places = written(&w.sandbox).expect("written");
    assert_eq!(places.cloud.len(), 1);
    assert_eq!(places.cloud[0].allowed, CloudAllowed::On);
}

#[test]
fn cancel_leaves_the_account_off_and_writes_nothing() {
    let mut w = world("assistants-cancel", vec![acme()], None);
    the_cloud_account_is_listed(&mut w);
    click(&mut w.harness, CLOUD);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Turn on me@acme.example?")
    });
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-answers=\"escape\"]",
    );
    settle(&mut w.harness, "the question to go", |h| {
        !h.html().contains("Turn on me@acme.example?")
    });
    assert_eq!(written(&w.sandbox), None);
    assert_eq!(
        w.harness.attr(CLOUD, "aria-checked").as_deref(),
        Some("false")
    );
}

#[test]
fn a_computer_of_the_persons_switches_without_a_question_and_the_file_follows() {
    let file = "[assistant]\nthis_computer = {}\n\n[[assistant.own_computers]]\nname = \"lab server\"\nallowed = \"on\"\n";
    let mut w = world("assistants-own", vec![acme()], Some(file));
    settle(&mut w.harness, "the computer", |h| h.count(LAB) == 1);
    assert_eq!(w.harness.attr(LAB, "aria-checked").as_deref(), Some("true"));
    click(&mut w.harness, LAB);
    settle(&mut w.harness, "the write", |_| {
        written(&w.sandbox).is_some_and(|places| places.own[0].allowed == Toggle::Off)
    });
    assert!(!w.harness.html().contains("Turn on lab server?"));
}

#[test]
fn ask_me_each_time_sits_under_a_cloud_rows_advanced_and_writes_ask() {
    let file = "[assistant]\nthis_computer = {}\n\n[[assistant.cloud_accounts]]\nname = \"me@acme.example\"\nallowed = \"on\"\nprovider = \"Acme\"\n";
    let mut w = world("assistants-ask", vec![acme()], Some(file));
    the_cloud_account_is_listed(&mut w);
    assert_eq!(
        w.harness.attr(CLOUD, "aria-checked").as_deref(),
        Some("true")
    );
    click(
        &mut w.harness,
        ".ds-disclosure-trigger[aria-label=\"Advanced, me@acme.example\"]",
    );
    let ask = "[aria-label=\"Ask me each time\"] .ds-toggle";
    settle(&mut w.harness, "the switch", |h| h.count(ask) == 1);
    assert_eq!(
        w.harness.count(
            "[aria-label=\"Advanced, This computer\"] ~ * [aria-label=\"Ask me each time\"]"
        ),
        0,
        "this computer has no such switch"
    );
    click(&mut w.harness, ask);
    settle(&mut w.harness, "the write", |_| {
        written(&w.sandbox)
            .is_some_and(|places| places.cloud[0].allowed == CloudAllowed::AskEachTime)
    });
}

#[test]
fn an_assistant_that_signs_itself_in_keeps_its_own_row_apart_from_the_places() {
    let claude = Held::new("claude", "Claude Code", "needs_login", &["agent"])
        .in_group("agent")
        .signing_in("agent");
    let mut w = world("assistants-external", vec![acme(), claude], None);
    settle(&mut w.harness, "the assistants", |h| {
        h.count("[data-assistants=\"others\"]") == 1
    });
    let html = w.harness.html();
    let others = &html[html.find("data-assistants=\"others\"").expect("the group")..];
    assert!(others.contains("Claude Code") && others.contains("Needs sign-in"));
    assert!(!others.contains("me@acme.example"), "no combined list");
    click(&mut w.harness, "[aria-label=\"Sign in Claude Code\"]");
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
}
