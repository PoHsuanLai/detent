//! The banner at the head of an account that needs the person: its sign-in ended, or the provider
//! is about to end it. Over a fake accountd on a private bus.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held, Policy};
use crate::support::bus::PrivateBus;
use crate::support::drive::click;
use detent_model::PageSlug;
use detent_ui::{Opening, STYLE};
use ds_harness::Harness;
use ds_lint::{LintConfig, markup};
use std::time::Duration;

const BANNER: &str = "[data-accounts=\"attention\"]";
const BANNER_BUTTON: &str = "[data-accounts=\"attention\"] .ds-button";
const SIGN_IN_ROW: &str = "[aria-label=\"Sign in again\"] .ds-field-row-control .ds-button";
const EXPIRY_ROW: &str = "[aria-label=\"Sign-in\"]";
const ENDING: &str = "Google signs this account out on 2026-10-14";
const ENDED: &str = "Signed out by Google on 2026-10-14";

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fake daemon answers on a real thread; a loaded machine running every test at once can
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

/// A Google account with the given standing and expiry words.
fn google(id: &str, label: &str, state: &str, words: Option<&str>) -> Held {
    let held = Held::new(id, label, state, &["mail"]).of_provider("google", "Google");
    match words {
        Some(words) => held.expiring(words),
        None => held,
    }
}

/// The page over a fake accountd holding `rows`, with the account `id` chosen.
struct World {
    harness: Harness,
    fake: Accountd,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _served: (zbus::Connection, ds_settings::live::Served),
}

fn world(name: &str, rows: Vec<Held>, chosen: &str) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon");
    let bus = PrivateBus::start(name);
    let count = rows.len();
    let fake = Accountd::new(Policy::Allow, rows);
    let served = rt.block_on(fake.serve_announcing(&bus));
    let opening = Opening {
        page: Some(PageSlug("accounts".to_owned())),
        query: String::new(),
    };
    let mut harness = Sandbox::new(name)
        .on_bus(&bus.address)
        .harness_in(opening, ds::prelude::Scheme::Light);
    settle(&mut harness, "the list", |h| {
        h.count(".account-row-body") >= count
    });
    click(
        &mut harness,
        &format!("#account-{}", chosen.replace('.', "-")),
    );
    settle(&mut harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some(chosen)
    });
    World {
        harness,
        fake,
        _rt: rt,
        _bus: bus,
        _served: served,
    }
}

#[test]
fn a_sign_in_that_ended_says_so_in_a_banner_that_replaces_the_rows() {
    let rows = vec![google(
        "work.example",
        "Work mail",
        "needs_reauth",
        Some(ENDED),
    )];
    let w = world("banner-ended", rows, "work.example");
    assert_eq!(w.harness.count(BANNER), 1, "{}", w.harness.html());
    assert!(
        w.harness
            .html()
            .contains("Sign in again to keep using Work mail."),
        "{}",
        w.harness.html()
    );
    assert_eq!(
        w.harness.count(SIGN_IN_ROW),
        0,
        "the row moved into the banner"
    );
    assert_eq!(w.harness.count(EXPIRY_ROW), 0, "one message, not two");
    assert_eq!(
        w.harness.attr(BANNER_BUTTON, "aria-label").as_deref(),
        Some("Sign In\u{2026} Work mail")
    );
    assert_eq!(
        w.harness
            .count("#account-work-example [aria-label=\"Needs sign-in\"]"),
        1,
        "the list row carries the dot"
    );
}

#[test]
fn a_sign_in_the_provider_will_end_names_the_provider_and_the_day() {
    let rows = vec![google("work.example", "Work mail", "ok", Some(ENDING))];
    let w = world("banner-ending", rows, "work.example");
    assert_eq!(w.harness.count(BANNER), 1, "{}", w.harness.html());
    assert!(
        w.harness
            .html()
            .contains("Google will sign this account out on 14 Oct."),
        "{}",
        w.harness.html()
    );
    assert_eq!(w.harness.count(SIGN_IN_ROW), 0);
    assert_eq!(w.harness.count(EXPIRY_ROW), 0);
    assert_eq!(
        w.harness
            .count("#account-work-example [aria-label=\"Sign-in ends soon\"]"),
        1,
        "the list row carries the dot"
    );
}

#[test]
fn an_account_that_needs_nothing_keeps_its_row_and_has_no_banner() {
    let rows = vec![google("home", "Home cloud", "ok", None)];
    let w = world("banner-none", rows, "home");
    assert_eq!(w.harness.count(BANNER), 0);
    assert_eq!(w.harness.count(SIGN_IN_ROW), 1);
    assert_eq!(
        w.harness
            .count("#account-home [aria-label=\"Needs sign-in\"]"),
        0
    );
    assert_eq!(w.harness.count("#account-home .ds-badge"), 0);
}

#[test]
fn the_banners_button_starts_the_sign_in_in_accountd_for_either_banner() {
    for (name, state, words) in [
        ("banner-press-ended", "needs_reauth", ENDED),
        ("banner-press-ending", "ok", ENDING),
    ] {
        let rows = vec![google("work.example", "Work mail", state, Some(words))];
        let mut w = world(name, rows, "work.example");
        assert!(w.fake.sets().is_empty(), "{name}: nothing was set yet");
        click(&mut w.harness, BANNER_BUTTON);
        settle(&mut w.harness, "the action", |_| !w.fake.sets().is_empty());
        assert_eq!(
            w.fake.sets(),
            [(
                "accounts.work.example.reauth".to_owned(),
                toml::Value::Boolean(true)
            )],
            "{name}"
        );
    }
}

#[test]
fn the_banner_is_only_quire_components_and_layout_only_css() {
    let rows = vec![google(
        "work.example",
        "Work mail",
        "needs_reauth",
        Some(ENDED),
    )];
    let w = world("banner-lint", rows, "work.example");
    assert_eq!(w.harness.count(BANNER), 1);
    let css = format!("{}\n{STYLE}", ds::stylesheet());
    let offences = markup(&w.harness.html(), &css, &LintConfig::new(&ds::kits()));
    assert!(offences.is_empty(), "{offences:#?}");
}
