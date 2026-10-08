//! "On this computer" on the Intelligence page, over a fake accountd on a private bus: the local
//! runtimes it lists, a stopped one greyed, each linking to its account in the Accounts pane.

mod support;

use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::Scheme;
use ds_harness::{Backdrop, Driver, Harness, Query};
use std::path::PathBuf;
use std::time::Duration;
use support::Sandbox;
use support::accountd::{Accountd, Held, Policy};
use support::bus::PrivateBus;
use support::drive::{click, page_of, scroll_into_view};

const ROWS: &str = ".runtimes .ds-field-row";
const OLLAMA: &str = ".runtimes [aria-label=\"Ollama\"] .ds-button";
const LM_STUDIO: &str = ".runtimes [aria-label=\"LM Studio\"] .ds-button";

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

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon")
}

/// Two runtimes on this computer (one stopped) and two accounts that are not.
fn held() -> Vec<Held> {
    vec![
        Held::runtime("ollama", "Ollama", "ok"),
        Held::runtime("lm-studio", "LM Studio", "offline"),
        Held::new("anthropic", "Anthropic", "ok", &["llm"]),
        Held::new("home", "Home cloud", "ok", &["storage"]),
    ]
}

struct World {
    fake: Accountd,
    harness: Harness,
    daemon: zbus::Connection,
    served: ds_settings::live::Served,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
}

fn world_in(name: &str, scheme: Scheme, accounts: Vec<Held>) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let fake = Accountd::new(Policy::Allow, accounts);
    let (daemon, served) = rt.block_on(fake.serve_announcing(&bus));
    let mut harness = Sandbox::new(name)
        .on_bus(&bus.address)
        .harness_in(intelligence(), scheme);
    settle(&mut harness, "the runtimes", |h| {
        h.attr(".runtimes", "data-runtimes").as_deref() == Some("list")
    });
    World {
        fake,
        harness,
        daemon,
        served,
        _rt: rt,
        _bus: bus,
    }
}

fn world(name: &str) -> World {
    world_in(name, Scheme::Light, held())
}

fn states(harness: &Harness) -> Vec<Option<String>> {
    (1..=harness.count(ROWS))
        .map(|n| harness.attr(&format!("{ROWS}:nth-child({n})"), "data-runtime"))
        .collect()
}

#[test]
fn the_runtimes_are_listed_between_the_models_switch_and_the_picker_running_or_not() {
    let w = world("runtimes-list");
    assert_eq!(
        w.harness.count(ROWS),
        2,
        "no cloud account, no storage account"
    );
    let html = w.harness.html();
    for text in [
        "On this computer",
        "Ollama",
        "LM Studio",
        "Running",
        "Offline",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert!(!html.contains("Anthropic") && !html.contains("Home cloud"));
    let at = |text: &str| html.find(text).unwrap_or_else(|| panic!("{text}"));
    assert!(at("Keep AI on this computer") < at("On this computer"));
    assert!(at(">On this computer<") < at("Model choice"));
}

#[test]
fn a_stopped_runtime_is_greyed_with_its_state_and_still_opens() {
    let w = world("runtimes-offline");
    assert_eq!(
        states(&w.harness),
        [Some("running".to_owned()), Some("offline".to_owned())]
    );
    let muting = |n: usize| {
        w.harness
            .attr(&format!("{ROWS}:nth-child({n}) .ds-avatar"), "data-muting")
    };
    assert_eq!(
        muting(1),
        None,
        "a running runtime's mark is at full strength"
    );
    assert_eq!(muting(2).as_deref(), Some("muted"));
    assert_eq!(
        w.harness.count(LM_STUDIO),
        1,
        "a stopped runtime keeps its link"
    );
}

#[test]
fn a_runtime_that_goes_offline_is_greyed_when_accountd_says_a_row_changed() {
    let mut w = world("runtimes-going-offline");
    w.fake.stop("ollama", "offline");
    w._rt
        .block_on(w.served.changed(
            &ds_settings::schema::KeyPath("accounts.ollama.state".to_owned()),
            &toml::Value::String("offline".to_owned()),
        ))
        .expect("announced");
    settle(&mut w.harness, "the row greying", |h| {
        states(h) == [Some("offline".to_owned()), Some("offline".to_owned())]
    });
    let _keep = &w.daemon;
}

#[test]
fn managing_a_runtime_opens_its_account_in_the_accounts_pane() {
    let mut w = world("runtimes-manage");
    scroll_into_view(&mut w.harness, OLLAMA);
    click(&mut w.harness, OLLAMA);
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
    assert_eq!(page_of(&w.harness).as_deref(), Some("accounts"));
    let html = w.harness.html();
    assert!(html.contains("Ollama") && !html.contains("LM Studio"));
}

#[test]
fn an_accountd_that_names_no_place_lists_no_runtime() {
    let name = "runtimes-none";
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let fake = Accountd::new(
        Policy::Allow,
        vec![Held::new("ollama", "Ollama", "ok", &["llm"])],
    );
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = Sandbox::new(name)
        .on_bus(&bus.address)
        .harness(intelligence());
    settle(&mut harness, "the empty list", |h| {
        h.attr(".runtimes", "data-runtimes").as_deref() == Some("empty")
    });
    assert!(harness.html().contains("Nothing is running here yet"));
}

#[test]
fn without_accountd_the_list_says_so_and_the_page_still_works() {
    let sandbox = Sandbox::new("runtimes-no-bus");
    let harness = sandbox.harness(intelligence());
    assert_eq!(
        harness.attr(".runtimes", "data-runtimes").as_deref(),
        Some("unavailable")
    );
    assert!(harness.html().contains("Keep AI on this computer"));

    let rt = runtime();
    let bus = PrivateBus::start("runtimes-no-accountd");
    let mut harness = Sandbox::new("runtimes-no-accountd")
        .on_bus(&bus.address)
        .harness(intelligence());
    settle(&mut harness, "accountd given up on", |h| {
        h.html().contains("The accounts service is not running")
    });
    drop(rt);
}

fn shot(harness: &mut Harness, name: &str) -> usize {
    let dir = std::env::var_os("DETENT_SHOTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(support::TMP).join("shots"));
    std::fs::create_dir_all(&dir).expect("shots dir");
    let image = harness.render_over(Backdrop::Scheme).expect("a picture");
    image.save(dir.join(name)).expect("save png");
    image
        .pixels()
        .map(|p| p.0)
        .collect::<std::collections::HashSet<_>>()
        .len()
}

#[test]
fn the_page_with_its_runtimes_is_drawn_in_light_and_dark() {
    for (scheme, tag) in [(Scheme::Light, "light"), (Scheme::Dark, "dark")] {
        let mut w = world_in(&format!("runtimes-shots-{tag}"), scheme, held());
        scroll_into_view(&mut w.harness, ROWS);
        assert!(shot(&mut w.harness, &format!("intelligence-{tag}.png")) > 50);
    }
}
