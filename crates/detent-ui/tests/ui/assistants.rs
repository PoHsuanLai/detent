//! The Intelligence page's assistants: where the built-in assistant may run (this computer, the
//! person's computers, signed-in cloud accounts), written to docket's settings file, and the
//! assistants that sign themselves in; and "Your computers" (the computers the assistant may borrow
//! models from, the Tailscale computers that can be added, the computers asking to borrow this
//! one's) over a fake inferd.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held, Policy};
use crate::support::bus::PrivateBus;
use crate::support::computers::{Asker, Fake, Lender, Own, serve_with};
use crate::support::drive::{click, click_when_still, scroll_into_view, type_into};
use crate::support::inferd::{Inferd, Policy as InferdPolicy};
use detent_model::PageSlug;
use detent_ui::Opening;
use docket_settings::{CloudAllowed, Toggle, read_places};
use ds_harness::{Harness, Input};
use std::time::Duration;

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

/// The page over a fake accountd holding `held`, in a world whose docket file holds `file`.
struct World {
    sandbox: Sandbox,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _daemon: zbus::Connection,
}

fn world(name: &str, held: Vec<Held>, file: Option<&str>) -> World {
    world_with(name, held, file, None)
}

/// The same, with docket's list of agents too.
fn world_with(name: &str, held: Vec<Held>, file: Option<&str>, agents: Option<&str>) -> World {
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
    if let Some(text) = agents {
        sandbox.write("docket/agents.toml", text);
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
    let file = "[assistant]\nthis_computer = {}\n\n[[assistant.own_computers]]\nname = \"lab-server\"\nallowed = \"on\"\n";
    let mut c = computers_in(
        "assistants-own",
        Some(file),
        vec![LAB_SERVER],
        vec![],
        vec![],
    );
    settle(&mut c.harness, "the computer", |h| h.count(LAB) == 1);
    assert_eq!(c.harness.attr(LAB, "aria-checked").as_deref(), Some("true"));
    click(&mut c.harness, LAB);
    settle(&mut c.harness, "the write", |_| {
        written(&c.sandbox).is_some_and(|places| places.own[0].allowed == Toggle::Off)
    });
    assert!(!c.harness.html().contains("Turn on lab-server?"));
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
    click_when_still(&mut w.harness, ask);
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
        h.count("[data-assistants=\"coding\"]") == 1
    });
    let html = w.harness.html();
    let others = &html[html.find("data-assistants=\"coding\"").expect("the group")..];
    assert!(others.contains("Claude Code") && others.contains("Needs sign-in"));
    assert!(!others.contains("me@acme.example"), "no combined list");
    click(&mut w.harness, "[aria-label=\"Sign in Claude Code\"]");
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
}

#[test]
fn an_assistant_docket_and_accountd_both_know_is_one_row_with_both_facts() {
    let claude = Held::new("claude", "Claude Code", "needs_login", &["agent"])
        .in_group("agent")
        .signing_in("agent");
    let mut w = world_with("assistants-joined", vec![claude], None, Some(AGENTS));
    let group = "[data-assistants=\"coding\"]";
    // The row is there from docket's list; accountd's status joins it once its list arrives.
    settle(&mut w.harness, "the status", |h| {
        h.text_of(group)
            .is_some_and(|t| t.contains("Needs sign-in"))
    });
    let text = w.harness.text_of(group).unwrap_or_default();
    assert_eq!(text.matches("Claude Code").count(), 1, "one row: {text}");
    for want in ["Needs sign-in", "The agent\u{2019}s own choice"] {
        assert!(text.contains(want), "{want} in {text}");
    }
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
        Some(Inferd::new(InferdPolicy::Allow).on_slots()),
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

#[test]
fn only_the_persons_own_computers_are_listed_and_there_is_no_address_to_type() {
    let c = computers("computers-list", vec![LAB_COMPUTER], vec![], vec![]);
    let text = group_text(&c.harness);
    for want in ["Your computers", "lab", "Qwen", "Ready", "Add\u{2026}"] {
        assert!(text.contains(want), "{want} in {text}");
    }
    for not in ["Acme One", "Local", "me@acme.example"] {
        assert!(
            !text.contains(not),
            "{not} is another kind of place: {text}"
        );
    }
    assert_eq!(c.harness.count(&format!("{GROUP} input")), 0);
    assert_eq!(
        c.fake.changes(),
        Vec::<String>::new(),
        "reading changes nothing"
    );
}

#[test]
fn add_lists_the_lenders_and_adding_one_moves_it_into_the_list() {
    let mut c = computers("computers-add", vec![LAB_COMPUTER], vec![STUDIO], vec![]);
    press(&mut c.harness, "Add a computer");
    settle(&mut c.harness, "the lender", |h| {
        group_text(h).contains("Llama. Say yes on studio when it asks.")
    });
    press(&mut c.harness, "Add studio");
    settle(&mut c.harness, "studio in the list", |_| {
        c.fake.changes() == ["add n1"]
    });
    settle(&mut c.harness, "the empty Add list", |h| {
        group_text(h).contains("None of your other computers is lending its models")
    });
    let text = group_text(&c.harness);
    assert_eq!(
        text.matches("studio").count(),
        1,
        "one row, not two: {text}"
    );
}

const SERVER_SHEET: &str = "[data-system=\"add-server\"]";

#[test]
fn a_server_is_added_from_its_sheet_and_shows_in_the_list_with_remove() {
    let mut c = computers_with_models("computers-server", vec![LAB_COMPUTER]);
    press(&mut c.harness, "Add a server");
    settle(&mut c.harness, "the sheet", |h| h.count(SERVER_SHEET) == 1);
    let sheet = c.harness.text_of(SERVER_SHEET).unwrap_or_default();
    assert!(
        sheet.contains("The server must answer on this computer."),
        "{sheet}"
    );
    settle(&mut c.harness, "the models", |h| {
        h.count("#server-model") == 1
    });
    type_into(&mut c.harness, "#server-name", "studio");
    type_into(&mut c.harness, "#server-port", "8080");
    click(&mut c.harness, "#server-add");
    settle(&mut c.harness, "the server in the list", |h| {
        h.count(SERVER_SHEET) == 0 && group_text(h).contains("studio")
    });
    assert_eq!(
        c.fake.changes(),
        ["add_server studio qwen port=8080 key=no"]
    );
    assert_eq!(
        c.harness
            .count(&format!("{GROUP} .ds-button[aria-label=\"Remove studio\"]")),
        1
    );
}

#[test]
fn a_sheet_that_asks_for_nothing_sendable_tells_why_and_calls_nothing() {
    let mut c = computers_with_models("computers-server-bad", vec![LAB_COMPUTER]);
    press(&mut c.harness, "Add a server");
    settle(&mut c.harness, "the sheet", |h| h.count(SERVER_SHEET) == 1);
    click(&mut c.harness, "#server-add");
    settle(&mut c.harness, "the sentence", |h| {
        h.text_of(SERVER_SHEET)
            .unwrap_or_default()
            .contains("Give the server a name.")
    });
    assert_eq!(c.fake.changes(), Vec::<String>::new());
}

#[test]
fn a_refused_server_keeps_the_sheet_open_with_the_services_sentence() {
    let mut c = computers_with_models("computers-server-refused", vec![LAB_COMPUTER]);
    press(&mut c.harness, "Add a server");
    settle(&mut c.harness, "the sheet", |h| h.count(SERVER_SHEET) == 1);
    settle(&mut c.harness, "the models", |h| {
        h.count("#server-model") == 1
    });
    type_into(&mut c.harness, "#server-name", "studio");
    type_into(&mut c.harness, "#server-port", "8080");
    c.fake.refuse_next("NotAnswering");
    click(&mut c.harness, "#server-add");
    settle(&mut c.harness, "the sentence", |h| {
        h.text_of(SERVER_SHEET)
            .unwrap_or_default()
            .contains("The sentence for NotAnswering.")
    });
    assert_eq!(c.harness.count(SERVER_SHEET), 1);
}

#[test]
fn remove_asks_for_that_computer_and_it_leaves_the_list() {
    let mut c = computers("computers-remove", vec![LAB_COMPUTER], vec![], vec![]);
    press(&mut c.harness, "Remove lab");
    settle(&mut c.harness, "the row to go", |h| {
        !group_text(h).contains("Qwen")
    });
    assert_eq!(c.fake.changes(), ["remove lab"]);
    assert!(group_text(&c.harness).contains("Computers you add appear here"));
}

#[test]
fn a_guest_is_allowed_denied_and_forgotten() {
    let mut c = computers("computers-guests", vec![], vec![], vec![LAPTOP]);
    let asking = "Asking to use this computer's models";
    assert!(group_text(&c.harness).contains(asking));
    press(&mut c.harness, "Allow laptop");
    settle(&mut c.harness, "approved", |h| {
        group_text(h).contains("Can use this computer's models")
    });
    press(&mut c.harness, "Don\u{2019}t Allow laptop");
    settle(&mut c.harness, "denied", |h| {
        group_text(h).contains("Not allowed to use this computer's models")
    });
    press(&mut c.harness, "Forget laptop");
    settle(&mut c.harness, "forgotten", |h| {
        !group_text(h).contains("laptop")
    });
    assert_eq!(
        c.fake.changes(),
        ["answer n3 allow", "answer n3 deny", "forget n3"]
    );
}

#[test]
fn each_refusal_reads_as_a_sentence_and_changes_nothing() {
    // The service answers any change with the refusal it is told to, so one button provokes all four.
    let mut c = computers("computers-refused", vec![], vec![STUDIO], vec![]);
    press(&mut c.harness, "Add a computer");
    settle(&mut c.harness, "the lender", |h| {
        group_text(h).contains("Say yes on studio")
    });
    // The service's own sentence is what the person reads.
    const REFUSALS: &[&str] = &[
        "NotOnTailscale",
        "NotAnswering",
        "NotAsking",
        "TooManyAsking",
    ];
    for refusal in REFUSALS {
        let sentence = format!("The sentence for {refusal}.");
        c.fake.refuse_next(refusal);
        press(&mut c.harness, "Add studio");
        settle(&mut c.harness, refusal, |h| {
            h.html()
                .replace("&#39;", "'")
                .replace('\u{2019}', "'")
                .contains(&sentence)
        });
    }
    assert_eq!(
        c.fake.changes(),
        Vec::<String>::new(),
        "a refusal changes nothing"
    );
}

#[test]
fn without_inferd_the_group_says_so_and_nothing_else_is_blocked() {
    let bus = PrivateBus::start("computers-absent");
    let sandbox = Sandbox::new("computers-absent").on_bus(&bus.address);
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the notice", |h| {
        h.count("[data-computers=\"unavailable\"]") == 1
    });
    assert!(
        group_text_of(&harness, "[data-computers=\"unavailable\"]")
            .contains("The assistant is not running")
    );
}

fn group_text_of(harness: &Harness, selector: &str) -> String {
    harness.text_of(selector).unwrap_or_default()
}

#[test]
fn a_computer_that_starts_asking_appears_when_inferd_says_the_guests_changed() {
    let mut c = computers("computers-signal", vec![], vec![], vec![]);
    assert!(!group_text(&c.harness).contains("laptop"));
    c._rt.block_on(c.fake.asks(LAPTOP));
    settle(&mut c.harness, "the guest", |h| {
        group_text(h).contains("Asking to use this computer's models")
    });
    assert_eq!(
        c.fake.changes(),
        Vec::<String>::new(),
        "nobody was asked to change anything"
    );
}

// ---- Coding assistants: the outside agents docket may start ----

const AGENTS: &str = "# mine\n[[agent]]\nprogram = \"claude-code\"\nlabel = \"Claude Code\"\nregistry = \"claude-code\"\nversion = \"1.0.0\"\n";
const OFFERED: &str = "seen = 1\nstart = \"signed_in\"\n\n[[model]]\nid = \"sonnet\"\nname = \"Sonnet 5.5\"\n\n[[model]]\nid = \"opus\"\nname = \"Opus\"\n";

#[test]
fn an_outside_assistant_is_listed_by_its_plain_name_and_its_model_is_chosen_in_its_entry() {
    let sandbox = Sandbox::new("coding-assistants");
    sandbox.write("docket/agents.toml", AGENTS);
    sandbox.write_data("docket/agents/state/claude-code/offered.toml", OFFERED);
    let mut harness = sandbox.harness(intelligence());
    let group = "[data-assistants=\"coding\"]";
    settle(&mut harness, "the group", |h| h.count(group) == 1);
    let text = harness.text_of(group).unwrap_or_default();
    for want in [
        "Coding assistants",
        "Claude Code",
        "Signed in",
        "The agent\u{2019}s own choice",
    ] {
        assert!(text.contains(want), "{want} in {text}");
    }
    for protocol in ["ACP", "MCP", "protocol"] {
        assert!(!text.contains(protocol), "{protocol} in {text}");
    }
    assert_eq!(sandbox.read("docket/agents.toml").as_deref(), Some(AGENTS));

    let button = format!("{group} .ds-popup .ds-button");
    scroll_into_view(&mut harness, &button);
    click(&mut harness, &button);
    let item = ".ds-menu-item:nth-child(3) .ds-menu-label";
    let at = harness.centre(item).expect("the menu is open");
    harness.send(Input::pointer_move(at));
    click(&mut harness, item);
    let written = sandbox.read("docket/agents.toml").expect("the file");
    assert_eq!(
        written,
        format!("{AGENTS}model = \"opus\"\n"),
        "the model lands in the entry and the comment stays"
    );
}
