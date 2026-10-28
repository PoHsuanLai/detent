//! The Intelligence page's assistants: where the built-in assistant may run (this computer, the
//! person's computers, signed-in cloud accounts), written to docket's settings file, and the
//! assistants that sign themselves in; and "Your computers" (the computers the assistant may borrow
//! models from, the Tailscale computers that can be added, the computers asking to borrow this
//! one's) over a fake inferd.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held};
use crate::support::bus::PrivateBus;
use crate::support::computers::{Asker, Fake, Lender, Own, serve_with};
use crate::support::drive::{click, click_when_still, scroll_into_view, type_into};
use crate::support::inferd::Inferd;
use detent_model::PageSlug;
use detent_ui::Opening;
use docket_settings::{CloudAllowed, Toggle, read_places};
use ds_harness::{Harness, Input};
use ds_settings::live::Verdict;
use std::time::Duration;

mod agents;
mod cloud;
mod computers;

const CLOUD: &str = "[aria-label=\"me@acme.example\"] .ds-toggle";
const LAB: &str = "[aria-label=\"lab-server\"] .ds-toggle";
const SETTINGS: &str = "docket/settings.toml";

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

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

/// The page over a fake accountd module in memory holding `held`, in a world whose docket file holds `file`.
struct World {
    sandbox: Sandbox,
    harness: Harness,
}

fn world(name: &str, held: Vec<Held>, file: Option<&str>) -> World {
    world_with(name, held, file, None)
}

/// The same, with docket's list of agents too.
fn world_with(name: &str, held: Vec<Held>, file: Option<&str>, agents: Option<&str>) -> World {
    let fake = Accountd::new(Verdict::Allow, held);
    let sandbox = Sandbox::new(name).with_modules(fake.hosted());
    if let Some(text) = file {
        sandbox.write(SETTINGS, text);
    }
    if let Some(text) = agents {
        sandbox.write("docket/agents.toml", text);
    }
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the places", |h| {
        h.count("[data-assistants=\"places\"]") == 1
    });
    World { sandbox, harness }
}

fn acme() -> Held {
    Held::new("acme", "me@acme.example", "ok", &["llm"])
        .in_group("intelligence")
        .of_provider("acme", "Acme")
}

fn written(sandbox: &Sandbox) -> Option<docket_settings::Places> {
    sandbox.read(SETTINGS).map(|text| read_places(&text).places)
}

// ---- Your computers: the persons computers, the Add list, the guests ----

const GROUP: &str = "[data-computers=\"list\"]";
const LAB_COMPUTER: Own = Own {
    name: "lab",
    models: &[("qwen", "Qwen")],
    ready: true,
};
const STUDIO: Lender = Lender {
    node: "n1",
    name: "studio",
    models: &[("llama", "Llama")],
    needs_approval: true,
};
const LAPTOP: Asker = Asker {
    node: "n3",
    name: "laptop",
    state: "asking",
};

const LAB_SERVER: Own = Own {
    name: "lab-server",
    models: &[("qwen", "Qwen")],
    ready: true,
};

struct Computers {
    sandbox: Sandbox,
    harness: Harness,
    fake: Fake,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
}

fn computers(name: &str, own: Vec<Own>, lenders: Vec<Lender>, askers: Vec<Asker>) -> Computers {
    computers_in(name, None, own, lenders, askers)
}

/// The same, with inferd's model picker served beside the computers: the models the Add a server
/// sheet offers.
fn computers_with_models(name: &str, own: Vec<Own>) -> Computers {
    build(
        name,
        None,
        own,
        vec![],
        vec![],
        Some(Inferd::new(Verdict::Allow).on_slots()),
    )
}

/// The same, in a world whose docket file holds `file`.
fn computers_in(
    name: &str,
    file: Option<&str>,
    own: Vec<Own>,
    lenders: Vec<Lender>,
    askers: Vec<Asker>,
) -> Computers {
    build(name, file, own, lenders, askers, None)
}

fn build(
    name: &str,
    file: Option<&str>,
    own: Vec<Own>,
    lenders: Vec<Lender>,
    askers: Vec<Asker>,
    models: Option<Inferd>,
) -> Computers {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon");
    let bus = PrivateBus::start(name);
    let fake = rt.block_on(serve_with(&bus, own, lenders, askers, models.as_ref()));
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    if let Some(text) = file {
        sandbox.write(SETTINGS, text);
    }
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the computers", |h| h.count(GROUP) == 1);
    Computers {
        sandbox,
        harness,
        fake,
        _rt: rt,
        _bus: bus,
    }
}

/// The group's words, with the typographic apostrophe a sentence may carry made plain.
fn group_text(harness: &Harness) -> String {
    harness
        .text_of(GROUP)
        .unwrap_or_default()
        .replace('\u{2019}', "'")
}

fn press(harness: &mut Harness, aria: &str) {
    let selector = format!("{GROUP} .ds-button[aria-label=\"{aria}\"]");
    settle(harness, aria, |h| h.count(&selector) == 1);
    scroll_into_view(harness, &selector);
    click(harness, &selector);
}

// ---- Coding assistants: the outside agents docket may start ----

const AGENTS: &str = "# mine\n[[agent]]\nprogram = \"claude-code\"\nlabel = \"Claude Code\"\nregistry = \"claude-code\"\nversion = \"1.0.0\"\n";
const OFFERED: &str = "seen = 1\nstart = \"signed_in\"\n\n[[model]]\nid = \"sonnet\"\nname = \"Sonnet 5.5\"\n\n[[model]]\nid = \"opus\"\nname = \"Opus\"\n";
