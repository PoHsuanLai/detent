//! The Privacy & Security page over a fake compositor on a private bus: the programs it lists are
//! named from desktop entries, each power shows where it stands, a choice is sent as the one
//! change the compositor takes, and an administrator's answer is locked.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::compositor::{Compositor, Row};
use crate::support::drive::{click, scroll_into_view, sidebar_row};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::{Harness, Input};
use std::time::Duration;

const GRIM: &str = "/usr/bin/grim";
const OTHER: &str = "/opt/tools/capture";
const LOUPE: &str = "org.gnome.Loupe";

/// A power's row of a program: `privacy-<program>-<power>`, as the page names it.
fn line(program: &str, power: &str) -> String {
    format!("#privacy-{program}-{power}")
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fake daemon answers on real threads; a loaded machine running every test at once can
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

struct World {
    fake: Compositor,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _daemon: Option<(zbus::Connection, ds_settings::live::Served)>,
}

/// The page, with the compositor listing `rows` on a private bus; none when `rows` is `None`.
fn world(name: &str, rows: Option<Vec<Row>>) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon");
    let bus = PrivateBus::start(name);
    let fake = Compositor::holding(rows.clone().unwrap_or_default());
    let daemon = rows.map(|_| rt.block_on(fake.serve_on(&bus)));
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    sandbox.write_application(
        "grim",
        "[Desktop Entry]\nType=Application\nName=Grim\nExec=/usr/bin/grim %f\n",
    );
    sandbox.write_application(
        "loupe",
        "[Desktop Entry]\nType=Application\nName=Image Viewer\nExec=flatpak run org.gnome.Loupe\nX-Flatpak=org.gnome.Loupe\n",
    );
    let harness = sandbox.harness(Opening {
        page: Some(PageSlug("privacy".to_owned())),
        query: String::new(),
    });
    World {
        fake,
        harness,
        _rt: rt,
        _bus: bus,
        _daemon: daemon,
    }
}

fn listed(world: &mut World) {
    settle(&mut world.harness, "the list", |h| {
        h.count("[data-privacy=\"list\"]") == 1
    });
}

fn label_of(harness: &Harness, row: &str) -> Option<String> {
    harness
        .text_of(&format!("{row} .ds-field-row-control .ds-button-label"))
        .or_else(|| harness.text_of(&format!("{row} .ds-field-row-control .ds-label")))
        .map(|text| text.trim().to_owned())
}

/// Opens the row's pop-up and clicks the item `nth` (from 1), the way a person points at it first.
fn choose(harness: &mut Harness, row: &str, nth: usize) {
    let button = format!("{row} .ds-popup .ds-button");
    scroll_into_view(harness, &button);
    click(harness, &button);
    let item = format!(".ds-menu-item:nth-child({nth}) .ds-menu-label");
    let at = harness.centre(&item).expect("the menu is open");
    harness.send(Input::pointer_move(at));
    click(harness, &item);
}

fn answers() -> Vec<Row> {
    vec![
        Row::person("screen", "aaa", GRIM, "on"),
        Row::person("clipboard", "bbb", GRIM, "off"),
        Row::person("windows", "ccc", LOUPE, "on"),
        Row::administrator("keys", "ddd", OTHER, "off"),
    ]
}

#[test]
fn the_page_is_in_the_sidebar_right_after_the_assistant_model() {
    let mut w = world("privacy-sidebar", Some(answers()));
    assert_eq!(
        w.harness.text_of(&sidebar_row(6)).as_deref(),
        Some("Assistant Model")
    );
    assert_eq!(
        w.harness.text_of(&sidebar_row(7)).as_deref(),
        Some("Privacy & Security")
    );
    listed(&mut w);
}

#[test]
fn programs_are_named_from_their_entries_and_each_power_shows_where_it_stands() {
    let mut w = world("privacy-rows", Some(answers()));
    listed(&mut w);
    assert_eq!(w.harness.count("[data-program]"), 3);
    let names = w.harness.text_of(".privacy").unwrap_or_default();
    for name in ["Grim", "Image Viewer", "capture"] {
        assert!(names.contains(name), "{name} in {names}");
    }
    assert!(
        names.contains("In /opt/tools"),
        "a program no system entry names shows its folder: {names}"
    );
    let grim = "usr-bin-grim";
    // `(row, what it says)`.
    let cases = [
        (line(grim, "screen"), "Allowed"),
        (line(grim, "clipboard"), "Denied"),
        (line(grim, "keys"), "Ask"),
        (line("org-gnome-loupe", "windows"), "Allowed"),
        (line("opt-tools-capture", "keys"), "Denied"),
    ];
    for (row, want) in cases {
        assert_eq!(label_of(&w.harness, &row).as_deref(), Some(want), "{row}");
    }
    let page = w.harness.text_of(".privacy").unwrap_or_default();
    for power in [
        "Screen recording",
        "Keyboard input",
        "Typing help",
        "Clipboard",
        "Screen lock",
        "Displays",
        "Windows",
    ] {
        assert!(page.contains(power), "{power}");
    }
}

#[test]
fn a_choice_is_sent_as_the_one_change_and_the_list_follows() {
    let mut w = world("privacy-choose", Some(answers()));
    listed(&mut w);
    let screen = line("usr-bin-grim", "screen");
    // The menu: Allowed, Denied, Ask.
    choose(&mut w.harness, &screen, 2);
    settle(&mut w.harness, "the answer", |h| {
        label_of(h, &screen).as_deref() == Some("Denied")
    });
    assert_eq!(w.fake.log(), ["set privacy.screen.aaa off"]);

    let clipboard = line("usr-bin-grim", "clipboard");
    choose(&mut w.harness, &clipboard, 3);
    settle(&mut w.harness, "the question to come back", |h| {
        label_of(h, &clipboard).as_deref() == Some("Ask")
    });
    assert_eq!(
        w.fake.log(),
        [
            "set privacy.screen.aaa off",
            "forget privacy.clipboard.bbb.forget"
        ]
    );
}

#[test]
fn a_power_nobody_answered_is_switched_on_as_the_first_answer() {
    let rows = vec![
        Row::person("screen", "aaa", GRIM, "on"),
        Row::unanswered("keys", "bbb", GRIM),
        Row::unanswered("clipboard", "ccc", LOUPE),
    ];
    let mut w = world("privacy-first-answer", Some(rows));
    listed(&mut w);
    assert_eq!(w.harness.count("[data-program]"), 2);
    let keys = line("usr-bin-grim", "keys");
    assert_eq!(label_of(&w.harness, &keys).as_deref(), Some("Ask"));
    // The menu: Allowed, Denied, Ask.
    choose(&mut w.harness, &keys, 1);
    settle(&mut w.harness, "the first answer", |h| {
        label_of(h, &keys).as_deref() == Some("Allowed")
    });
    assert_eq!(w.fake.log(), ["set privacy.keys.bbb on"]);
    let clipboard = line("org-gnome-loupe", "clipboard");
    assert_eq!(label_of(&w.harness, &clipboard).as_deref(), Some("Ask"));
}

#[test]
fn an_administrators_answer_is_locked_and_says_so() {
    let mut w = world("privacy-floor", Some(answers()));
    listed(&mut w);
    let fixed = line("opt-tools-capture", "keys");
    assert_eq!(
        w.harness.attr(&fixed, "aria-disabled").as_deref(),
        Some("true")
    );
    assert_eq!(
        w.harness.count(&format!("{fixed} .ds-popup")),
        0,
        "nothing to pick"
    );
    let words = w.harness.text_of(&fixed).unwrap_or_default();
    assert!(words.contains("Set by your administrator"), "{words}");
    assert!(w.fake.log().is_empty());
}

#[test]
fn with_no_compositor_the_page_says_so_in_plain_words() {
    let mut w = world("privacy-absent", None);
    settle(&mut w.harness, "the line", |h| {
        h.count("[data-privacy=\"unavailable\"]") == 1
    });
    let words = w.harness.text_of(".privacy").unwrap_or_default();
    assert!(words.contains("The desktop is not running"), "{words}");
    for banned in ["compositor", "D-Bus", "bus"] {
        assert!(!words.contains(banned), "{banned} in {words}");
    }
}
