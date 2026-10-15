//! The assistant's model: its page in the sidebar and its first-run step, over a fake inferd on a
//! private bus. Nothing is chosen until the person chooses, and an unset model shows as such.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::drive::{click, scroll_into_view, sidebar_row};
use crate::support::inferd::{Inferd, Policy};
use detent_model::{ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, PageSlug};
use detent_ui::{Notice, Opening};
use ds_harness::{Driver, Harness, Input, Query};
use std::time::Duration;

const ROW: &str = "ai.model.text.balanced";
const BUTTON: &str = ".assistant-model .ds-popup .ds-button";
const SETUP_BUTTON: &str = ".assistant-setup .ds-popup .ds-button";
const CONTINUE: &str = ".assistant-continue";
const SKIP: &str = ".assistant-skip";

fn opening(slug: &str) -> Opening {
    Opening {
        page: Some(PageSlug(slug.to_owned())),
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

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon")
}

/// The row's button text, once the picker is there.
fn shown(harness: &Harness, button: &str) -> Option<String> {
    harness
        .text_of(&format!("{button} .ds-button-label"))
        .map(|text| text.trim().to_owned())
}

/// Opens the row's menu and returns the number of items marked chosen.
fn open_and_count_marked(harness: &mut Harness, button: &str) -> usize {
    scroll_into_view(harness, button);
    click(harness, button);
    assert_eq!(
        harness.attr(button, "aria-expanded").as_deref(),
        Some("true"),
        "the click opened the menu"
    );
    harness.count(".ds-menu-item[aria-checked=\"true\"]")
}

/// Clicks the menu item `nth` (from 1), the way a person points at it first.
fn pick(harness: &mut Harness, nth: usize) {
    let label = format!(".ds-menu-item:nth-child({nth}) .ds-menu-label");
    let at = harness.centre(&label).expect("the menu is open");
    harness.send(Input::pointer_move(at));
    click(harness, &label);
}

/// Inferd serving slots on a private bus, the sandbox looking at it, and the page's harness.
struct World {
    fake: Inferd,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _daemon: zbus::Connection,
}

fn world(name: &str, slug: &str, fake: Inferd) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let daemon = rt.block_on(fake.serve_on(&bus));
    let harness = Sandbox::new(name)
        .on_bus(&bus.address)
        .harness(opening(slug));
    World {
        fake,
        harness,
        _rt: rt,
        _bus: bus,
        _daemon: daemon,
    }
}

fn ready(world: &mut World) {
    settle(&mut world.harness, "the picker", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("ready")
    });
}

#[test]
fn the_page_follows_intelligence_in_the_sidebar_and_draws_only_the_assistants_row() {
    let mut world = world(
        "assistant-page",
        ASSISTANT_MODEL_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    // The fixtures' pages: Intelligence is the ninth, the page follows it.
    assert_eq!(
        world.harness.text_of(&sidebar_row(9)).as_deref(),
        Some("Intelligence")
    );
    assert_eq!(
        world.harness.text_of(&sidebar_row(10)).as_deref(),
        Some("Assistant Model"),
        "the sidebar lists the page after Intelligence"
    );
    assert_eq!(
        world
            .harness
            .attr(&sidebar_row(10), "aria-selected")
            .as_deref(),
        Some("true")
    );
    assert_eq!(
        world.harness.count(".assistant-model .ds-field-row"),
        1,
        "one row, not a card per slot and tier"
    );
    assert_eq!(world.harness.count(".picker-kind"), 0);
}

#[test]
fn a_model_nobody_chose_shows_a_prompt_and_marks_no_item_while_a_chosen_one_is_marked() {
    let mut empty = world(
        "assistant-unset",
        ASSISTANT_MODEL_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut empty);
    assert_eq!(
        shown(&empty.harness, BUTTON).as_deref(),
        Some("Choose\u{2026}"),
        "no model is claimed"
    );
    assert_eq!(
        open_and_count_marked(&mut empty.harness, BUTTON),
        0,
        "no row is selected"
    );
    let html = empty.harness.html();
    assert!(html.contains(">None<"), "the empty word reads None");
    for banned in ["Catalogue default", "Recommended", "recommended"] {
        assert!(!html.contains(banned), "{banned} is on the page:\n{html}");
    }

    let fake = Inferd::new(Policy::Allow).on_slots();
    fake.values
        .lock()
        .expect("values")
        .insert(ROW.to_owned(), "local/qwen".to_owned());
    let mut chosen = world("assistant-chosen", ASSISTANT_MODEL_SLUG, fake);
    ready(&mut chosen);
    assert_eq!(
        shown(&chosen.harness, BUTTON).as_deref(),
        Some("Local"),
        "a model that runs on this computer is the Local choice"
    );
    assert_eq!(
        open_and_count_marked(&mut chosen.harness, BUTTON),
        1,
        "the chosen one, and only it, is marked"
    );
}

#[test]
fn picking_a_model_sets_the_assistants_row_and_no_other() {
    let mut world = world(
        "assistant-pick",
        ASSISTANT_MODEL_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    open_and_count_marked(&mut world.harness, BUTTON);
    // Items: None, Automatic, Local, the Anthropic header, then claude.
    pick(&mut world.harness, 5);
    settle(&mut world.harness, "the set", |h| {
        shown(h, BUTTON).as_deref() == Some("claude")
    });
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        [(ROW.to_owned(), "anthropic/claude".to_owned())]
    );
}

#[test]
fn a_model_that_needs_an_account_stays_pickable_and_offers_to_add_one_instead_of_setting() {
    let fake = Inferd::new(Policy::Allow).on_slots();
    fake.gated.lock().expect("gated").insert(
        "anthropic/claude".to_owned(),
        "Add an account to use".to_owned(),
    );
    let mut world = world("assistant-account", ASSISTANT_MODEL_SLUG, fake);
    ready(&mut world);
    open_and_count_marked(&mut world.harness, BUTTON);
    // Items: None, Automatic, Local, the Anthropic header, then claude.
    let claude = ".ds-menu-item:nth-child(5)";
    assert_ne!(
        world.harness.attr(claude, "aria-disabled").as_deref(),
        Some("true"),
        "the choice is not greyed"
    );
    pick(&mut world.harness, 5);
    // No account service is on the bus, so the add sheet cannot open; the person is told calmly.
    settle(&mut world.harness, "the line", |h| {
        h.html()
            .contains("Could not add an account. Try again later.")
    });
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        Vec::<(String, String)>::new(),
        "nothing was set"
    );
    assert_eq!(
        shown(&world.harness, BUTTON).as_deref(),
        Some("Choose\u{2026}"),
        "and nothing is chosen"
    );
}

#[test]
fn the_first_run_step_has_no_sidebar_and_continue_waits_for_a_choice() {
    let mut world = world(
        "assistant-setup",
        ASSISTANT_SETUP_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    assert_eq!(world.harness.count(".assistant-setup"), 1);
    assert_eq!(
        world.harness.count(".ds-sidebar"),
        0,
        "a dialogue, not the window"
    );
    let html = world.harness.html();
    for text in ["Choose a Model for Your Assistant", "Skip", "Continue"] {
        assert!(html.contains(text), "{text} is on the step");
    }
    assert_eq!(
        shown(&world.harness, SETUP_BUTTON).as_deref(),
        Some("Choose\u{2026}")
    );
    assert_eq!(
        world.harness.attr(CONTINUE, "data-availability").as_deref(),
        Some("disabled"),
        "nothing is chosen, so Continue waits"
    );
    assert_ne!(
        world.harness.attr(SKIP, "data-availability").as_deref(),
        Some("disabled"),
        "Skip never waits"
    );

    open_and_count_marked(&mut world.harness, SETUP_BUTTON);
    pick(&mut world.harness, 5);
    settle(&mut world.harness, "Continue", |h| {
        h.attr(CONTINUE, "data-availability").as_deref() != Some("disabled")
    });
}

#[test]
fn skip_writes_nothing() {
    let mut world = world(
        "assistant-skip",
        ASSISTANT_SETUP_SLUG,
        Inferd::new(Policy::Allow).on_slots(),
    );
    ready(&mut world);
    click(&mut world.harness, SKIP);
    world.harness.advance(Duration::from_millis(300));
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        Vec::<(String, String)>::new(),
        "the model stays unset"
    );
}

#[test]
fn a_page_to_open_can_be_the_first_run_step() {
    let sandbox = Sandbox::new("assistant-notice");
    let (mut harness, outbox) = sandbox.harness_told(Opening::default());
    assert_eq!(harness.count(".assistant-setup"), 0);
    outbox
        .send(Notice::OpenPage(PageSlug(ASSISTANT_SETUP_SLUG.to_owned())))
        .expect("the window is listening");
    harness.advance(Duration::from_millis(400));
    assert_eq!(harness.count(".assistant-setup"), 1);
    assert_eq!(harness.count(".ds-sidebar"), 0);
    outbox
        .send(Notice::OpenPage(PageSlug(ASSISTANT_MODEL_SLUG.to_owned())))
        .expect("the window is listening");
    harness.advance(Duration::from_millis(400));
    assert_eq!(harness.count(".assistant-setup"), 0);
    assert_eq!(
        harness.attr(".scroll", "data-page").as_deref(),
        Some(ASSISTANT_MODEL_SLUG),
        "the window is back, on the page"
    );
}

const POOL_ROW: &str = "#pool-local-qwen .ds-toggle";
const POOL_NOTE: &str = "Pick at least one model for Automatic to choose from.";

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
    pick(&mut world.harness, 2);
    settle(&mut world.harness, "Automatic", |h| {
        shown(h, SETUP_BUTTON).as_deref() == Some("Automatic")
    });
    assert_eq!(
        *world.fake.sets.lock().expect("sets"),
        [(ROW.to_owned(), "auto".to_owned())]
    );
    assert_eq!(world.harness.count("[data-pool=\"editor\"]"), 1);
    assert!(
        world.harness.html().contains(POOL_NOTE),
        "an empty list says what to do"
    );
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
    assert!(world.harness.html().contains("Built in"));
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
