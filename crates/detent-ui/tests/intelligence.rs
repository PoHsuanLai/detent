//! The Intelligence page: its sections in design/22's order, inferd's model picker over a fake
//! `org.quire.SettingsModule1` on a private bus, and the page without inferd.

mod support;

use detent_model::PageSlug;
use detent_ui::{Opening, ReasonFeed};
use ds_harness::{Driver, Harness, Input, Query};
use porter_core::{AccountId, ModelId};
use porter_infer::{Declined, DeclinedBecause, InferEvent, ModelRef, Why};
use std::time::Duration;
use support::Sandbox;
use support::bus::PrivateBus;
use support::drive::{click, scroll_into_view};
use support::inferd::{Inferd, Policy};

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

/// A one-key schema of a program that has none of its own in the fixtures yet, written the way
/// design/22 section 3 names its keys.
fn stand_in(app: &str, file: &str, path: &str, section: &str) -> String {
    format!(
        "app = \"{app}\"\nfile = \"{file}\"\nversion = 1\n\n[[key]]\npath = \"{path}\"\n\
         default = \"on\"\nlabel = \"{path}\"\nhelp = \"\"\nsection = \"{section}\"\n\
         exposure = \"basic\"\n\n[key.kind]\nkind = \"toggle\"\n\n[key.kind.v]\n\
         variants = [\"on\", \"off\"]\n\n[key.page]\nkind = \"intelligence\"\n"
    )
}

/// Runs the page's async work: advance the document until `done` holds.
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

/// Positions of each of `texts` in `html`, each searched for after the one before.
fn in_order(html: &str, texts: &[&str]) -> Vec<usize> {
    let mut from = 0;
    texts
        .iter()
        .map(|text| {
            let at = html[from..]
                .find(&format!(">{text}<"))
                .unwrap_or_else(|| panic!("{text:?} is not on the page after byte {from}"));
            from += at;
            from
        })
        .collect()
}

fn model(name: &str) -> ModelRef {
    ModelRef {
        account: AccountId::parse("local").expect("account"),
        model: ModelId::parse(name).expect("model"),
    }
}

#[test]
fn the_sections_follow_design_order_whatever_order_the_programs_were_read_in() {
    let sandbox = Sandbox::new("intelligence-order");
    // Memory and Computer use come from almanac's and cua's own schemas, Privacy from docket's;
    // only sill's Companion section has no schema yet, so a stand-in names it.
    sandbox.add_schema(
        "sill-companion",
        &stand_in(
            "sill-companion",
            "sill/companion.toml",
            "companion.summon",
            "Companion",
        ),
    );
    let harness = sandbox.harness(intelligence());
    in_order(
        &harness.html(),
        &[
            "Companion",
            "Models",
            "Model choice",
            "Memory",
            "Computer use",
            "Privacy",
            "Advanced",
        ],
    );
}

#[test]
fn the_page_shows_inferds_and_dockets_file_rows_with_no_inferd_running() {
    let sandbox = Sandbox::new("intelligence-absent");
    let harness = sandbox.harness(intelligence());
    let html = harness.html();
    for row in [
        "Keep AI on this computer",
        "Mail",
        "Let other apps use my apps&#39; actions (MCP)",
    ] {
        assert!(
            html.contains(row) || html.contains(&row.replace("&#39;", "'")),
            "{row}"
        );
    }
    assert_eq!(
        harness.attr(".picker", "data-picker").as_deref(),
        Some("unavailable"),
        "no bus: the picker is unavailable, the page is not"
    );
    assert!(html.contains("Unavailable"));
    assert_eq!(harness.count(".picker-kind"), 0);
}

#[test]
fn an_absent_inferd_blocks_nothing_on_the_page() {
    let sandbox = Sandbox::new("intelligence-absent-writes");
    let mut harness = sandbox.harness(intelligence());
    assert_eq!(sandbox.read("quire/inferd.toml"), None);
    let local_only = "[aria-label=\"Keep AI on this computer\"] .ds-toggle";
    let before = harness.attr(local_only, "aria-checked");
    assert_eq!(before.as_deref(), Some("true"), "local-only starts on");

    click(&mut harness, local_only);
    assert_eq!(
        harness.attr(local_only, "aria-checked").as_deref(),
        Some("false")
    );
    let written = sandbox
        .read("quire/inferd.toml")
        .expect("the file row wrote");
    assert!(written.contains("local_only = \"off\""), "{written}");
}

#[test]
fn a_bus_with_no_inferd_on_it_says_inferd_is_not_running() {
    let sandbox = Sandbox::new("intelligence-no-inferd");
    let rt = runtime();
    let bus = PrivateBus::start("no-inferd");
    let sandbox = sandbox.on_bus(&bus.address);
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the picker giving up", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("inferd is not running"));
    drop(rt);
}

#[test]
fn inferd_refusing_settings_is_said_in_its_own_words() {
    let sandbox = Sandbox::new("intelligence-refused");
    let rt = runtime();
    let bus = PrivateBus::start("refused");
    let fake = Inferd::new(Policy::Refuse);
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the refusal", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("does not list Settings"));
}

#[test]
fn the_picker_lists_a_group_per_kind_and_a_row_per_tier_in_design_words() {
    let sandbox = Sandbox::new("intelligence-picker");
    let rt = runtime();
    let bus = PrivateBus::start("picker");
    let fake = Inferd::new(Policy::Allow);
    fake.values
        .lock()
        .expect("values")
        .insert("ai.model.llm.balanced".to_owned(), "local/qwen".to_owned());
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);

    assert_eq!(
        harness.count(".picker-kind .ds-field-row"),
        4,
        "3 language tiers + 1 computer-use tier"
    );
    let html = harness.html();
    in_order(
        &html,
        &["Language", "Fast", "Balanced", "Demanding", "Computer use"],
    );
    assert!(
        !html.contains(">Best<"),
        "the Best slot is called Demanding"
    );
    assert_eq!(
        harness
            .text_of(".picker-kind .ds-field-row:nth-child(2) .ds-button-label")
            .as_deref()
            .map(str::trim),
        Some("qwen"),
        "the module's current value is the one shown"
    );
}

#[test]
fn choosing_a_model_shows_loading_it_until_inferd_answers_then_sets_it() {
    let sandbox = Sandbox::new("intelligence-choose");
    let rt = runtime();
    let bus = PrivateBus::start("choose");
    let fake = Inferd::new(Policy::Allow);
    *fake.held.lock().expect("held") = true;
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);

    let fast = ".picker-kind .ds-field-row:nth-child(1)";
    let button = format!("{fast} .ds-popup .ds-button");
    scroll_into_view(&mut harness, &button);
    click(&mut harness, &button);
    assert_eq!(
        harness.attr(&button, "aria-expanded").as_deref(),
        Some("true"),
        "the click opened the menu"
    );
    let item = harness
        .centre(".ds-menu-item:nth-child(4) .ds-menu-label")
        .expect("the menu is open");
    harness.send(Input::pointer_move(item));
    click(&mut harness, ".ds-menu-item:nth-child(4) .ds-menu-label");
    settle(&mut harness, "Loading holo", |h| {
        h.html().contains("Loading holo")
    });
    assert_eq!(
        *fake.sets.lock().expect("sets"),
        [("ai.model.llm.fast".to_owned(), "local/holo".to_owned())],
        "exactly one Set, naming the row and the model"
    );
    assert_eq!(
        fake.values.lock().expect("values").get("ai.model.llm.fast"),
        None,
        "inferd has not put it in force yet"
    );

    fake.release.notify_one();
    settle(&mut harness, "the set to finish", |h| {
        !h.html().contains("Loading holo")
    });
    assert_eq!(
        fake.values
            .lock()
            .expect("values")
            .get("ai.model.llm.fast")
            .map(String::as_str),
        Some("local/holo")
    );
    assert_eq!(
        harness
            .text_of(&format!("{fast} .ds-button-label"))
            .as_deref()
            .map(str::trim),
        Some("holo")
    );
}

#[test]
fn the_footer_says_why_when_it_is_on_and_is_gone_when_it_is_off() {
    let rt = runtime();
    for (name, setting, shown) in [("on", None, true), ("off", Some("off"), false)] {
        let sandbox = Sandbox::new(&format!("intelligence-reason-{name}"));
        if let Some(word) = setting {
            sandbox.write(
                "quire/inferd.toml",
                &format!("[ai.auto]\nshow_reason = \"{word}\"\n"),
            );
        }
        let bus = PrivateBus::start(&format!("reason-{name}"));
        let fake = Inferd::new(Policy::Allow);
        let _daemon = rt.block_on(fake.serve_on(&bus));
        let (events, feed) = ReasonFeed::channel();
        let mut harness = sandbox
            .with_reasons(feed)
            .on_bus(&bus.address)
            .harness(intelligence());
        settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);
        assert_eq!(
            harness.count(".picker-reason"),
            usize::from(shown),
            "{name}"
        );

        events.send(InferEvent::Why(Why::Warm)).expect("sent");
        if shown {
            settle(&mut harness, "the reason", |h| {
                h.text_of(".picker-reason")
                    .is_some_and(|t| t.contains("Already running."))
            });
            events
                .send(InferEvent::Declined(Declined {
                    model: model("holo"),
                    because: DeclinedBecause::NotInstalled,
                }))
                .expect("sent");
            settle(&mut harness, "the refusal", |h| {
                h.text_of(".picker-reason").is_some_and(|t| {
                    t.contains("holo can't do this: it isn't on this computer yet.")
                })
            });
        } else {
            harness.advance(Duration::from_millis(200));
            assert_eq!(
                harness.count(".picker-reason"),
                0,
                "off: no footer whatever arrives"
            );
            assert!(!harness.html().contains("Already running."));
        }
    }
}

#[test]
fn an_inferd_that_starts_after_the_page_opened_is_found_and_one_that_stops_is_missed() {
    let sandbox = Sandbox::new("intelligence-live");
    let rt = runtime();
    let bus = PrivateBus::start("live");
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the card", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("inferd is not running"));
    assert_eq!(harness.count(".picker-kind"), 0);

    let fake = Inferd::new(Policy::Allow);
    let daemon = rt.block_on(fake.serve_on(&bus));
    settle(&mut harness, "the picker to appear", |h| {
        h.count(".picker-kind") == 2
    });
    assert_eq!(harness.count(".picker-kind .ds-field-row"), 4);

    drop(daemon);
    settle(&mut harness, "the card to return", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("inferd is not running"));
    assert_eq!(harness.count(".picker-kind"), 0);

    let _again = rt.block_on(fake.serve_on(&bus));
    settle(&mut harness, "the picker to come back", |h| {
        h.count(".picker-kind") == 2
    });
}

#[test]
fn watching_for_inferd_never_starts_it() {
    let sandbox = Sandbox::new("intelligence-never-activates");
    let rt = runtime();
    let bus = PrivateBus::start("never-activates");
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the card", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    let names = rt.block_on(async {
        let connection = bus.connect().await;
        zbus::fdo::DBusProxy::new(&connection)
            .await
            .expect("proxy")
            .list_names()
            .await
            .expect("names")
    });
    assert!(
        !names
            .iter()
            .any(|n| n.as_str() == porter_dbus::INFERENCE_BUS),
        "nothing owns inferd's name: the page only looked"
    );
}

/// Opens the picker's first fast row's menu.
fn open_first_menu(harness: &mut Harness) -> String {
    let button = ".picker-kind .ds-field-row:nth-child(1) .ds-popup .ds-button";
    scroll_into_view(harness, button);
    click(harness, button);
    assert_eq!(
        harness.attr(button, "aria-expanded").as_deref(),
        Some("true"),
        "the click opened the menu"
    );
    button.to_owned()
}

#[test]
fn rows_render_from_the_schemas_sections_and_labels_with_no_local_table() {
    let sandbox = Sandbox::new("intelligence-slots");
    let rt = runtime();
    let bus = PrivateBus::start("slots");
    let fake = Inferd::new(Policy::Allow).on_slots();
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 3);
    assert_eq!(harness.count(".picker-kind .ds-field-row"), 5);
    assert_eq!(
        harness.count(".picker-area"),
        1,
        "one Models title over every slot"
    );
    in_order(
        &harness.html(),
        &[
            "Models",
            "Text",
            "Fast",
            "Balanced",
            "Demanding",
            "Speech to text",
            "Fast",
            "Computer use",
            "Balanced",
        ],
    );
}

#[test]
fn grouped_choices_render_under_headers_ungrouped_first_in_first_appearance_order() {
    let sandbox = Sandbox::new("intelligence-groups");
    let rt = runtime();
    let bus = PrivateBus::start("groups");
    let fake = Inferd::new(Policy::Allow).on_slots();
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 3);
    open_first_menu(&mut harness);
    in_order(
        &harness.html(),
        &[
            "Catalogue default",
            "Automatic",
            "On this computer",
            "qwen",
            "holo",
            "Anthropic",
            "claude",
            "Google",
            "gemini",
        ],
    );
}

#[test]
fn an_old_key_module_still_renders() {
    let sandbox = Sandbox::new("intelligence-old-keys");
    let rt = runtime();
    let bus = PrivateBus::start("old-keys");
    let fake = Inferd::new(Policy::Allow);
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 2);
    assert_eq!(harness.count(".picker-area"), 1, "the section is the area");
    in_order(
        &harness.html(),
        &["Models", "Language", "Fast", "Computer use"],
    );
    open_first_menu(&mut harness);
    let html = harness.html();
    assert!(
        !html.contains("On this computer"),
        "a module with no groups has no headers:\n{html}"
    );
}

#[test]
fn an_unavailable_choice_is_greyed_with_its_reason_and_cannot_be_set() {
    let sandbox = Sandbox::new("intelligence-unavailable");
    let rt = runtime();
    let bus = PrivateBus::start("unavailable");
    let fake = Inferd::new(Policy::Allow).on_slots();
    fake.gated
        .lock()
        .expect("gated")
        .insert("local/holo".to_owned(), "Add an account to use".to_owned());
    let _daemon = rt.block_on(fake.serve_on(&bus));
    let mut harness = sandbox.on_bus(&bus.address).harness(intelligence());
    settle(&mut harness, "the picker", |h| h.count(".picker-kind") == 3);
    open_first_menu(&mut harness);

    let html = harness.html();
    assert!(
        html.contains("Add an account to use"),
        "the reason is under the choice:\n{html}"
    );
    // Items: "", auto, header, qwen, holo (disabled), its reason, then the other groups.
    let holo = ".ds-menu-item:nth-child(5)";
    assert_eq!(
        harness.attr(holo, "aria-disabled").as_deref(),
        Some("true"),
        "holo is greyed"
    );
    if let Some(centre) = harness.centre(&format!("{holo} .ds-menu-label")) {
        harness.send(Input::pointer_move(centre));
        click(&mut harness, &format!("{holo} .ds-menu-label"));
    }
    harness.advance(Duration::from_millis(300));
    assert_eq!(
        *fake.sets.lock().expect("sets"),
        Vec::<(String, String)>::new(),
        "a greyed choice sends no Set"
    );
}
