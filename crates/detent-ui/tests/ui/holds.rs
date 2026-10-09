//! A deletion the sync service holds for the person's word, on the account's sync row: over a fake
//! accountd and a fake syncd on a private bus.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held, Policy};
use crate::support::bus::PrivateBus;
use crate::support::drive::click;
use crate::support::syncd::Syncd;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::{Driver, Harness, Query};
use std::time::Duration;

const HOME: &str = "[aria-label=\"Manage Home cloud\"]";
const BANNER: &str = ".accounts-hold .ds-inline-banner";
const KEEP: &str = ".accounts-hold [data-answers=\"return\"]";
const REMOVE: &str = ".accounts-hold [data-role=\"destructive\"]";
const KEEP_FILES: &str = "Keep this account's files on this computer";

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

struct World {
    harness: Harness,
    sync: Syncd,
    sync_bus: zbus::Connection,
    rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _accounts: zbus::Connection,
    _served: ds_settings::live::Served,
    _fake: Accountd,
}

/// The Accounts page over the fakes, with `before` holds already on when the window opens.
fn world(name: &str, before: &[(&str, u64, u64)]) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemons");
    let bus = PrivateBus::start(name);
    let fake = Accountd::new(
        Policy::Allow,
        vec![Held::new("home", "Home cloud", "ok", &["storage"]).syncing()],
    );
    let (accounts, served) = rt.block_on(fake.serve_announcing(&bus));
    let sync = Syncd::default();
    let sync_bus = rt.block_on(sync.serve_on(&bus));
    for (dataset, discard, held) in before {
        rt.block_on(sync.hold(&sync_bus, dataset, *discard, *held));
    }
    let opening = Opening {
        page: Some(PageSlug("accounts".to_owned())),
        query: String::new(),
    };
    let mut harness = Sandbox::new(name).on_bus(&bus.address).harness(opening);
    settle(&mut harness, "the list", |h| h.count(".accounts-list") == 1);
    click(&mut harness, HOME);
    settle(&mut harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
    World {
        harness,
        sync,
        sync_bus,
        rt,
        _bus: bus,
        _accounts: accounts,
        _served: served,
        _fake: fake,
    }
}

#[test]
fn a_hold_told_after_the_window_opened_shows_on_the_sync_row_with_keep_as_the_default() {
    let mut w = world("holds-live", &[]);
    assert_eq!(w.harness.count(BANNER), 0);
    w.rt.block_on(w.sync.hold(&w.sync_bus, "home/files", 30, 40));
    settle(&mut w.harness, "the banner", |h| h.count(BANNER) == 1);
    let html = w.harness.html();
    assert!(
        html.contains("30 of 40 files were removed on the server. Remove them here too?"),
        "{html}"
    );
    assert!(html.contains("Keep the files here. You will be asked again if they are still gone."));
    assert_eq!(w.harness.count(KEEP), 1, "Keep answers Return");
    assert_eq!(w.harness.count(REMOVE), 1, "Remove is destructive");
    assert_eq!(
        w.harness
            .count(".accounts-hold [data-role=\"destructive\"][data-answers=\"return\"]"),
        0
    );
    let row = html.find(KEEP_FILES).expect("the sync row");
    let banner = html.find("30 of 40").expect("the banner");
    assert!(row < banner, "the banner is under the row it is about");
}

#[test]
fn a_hold_already_on_when_the_window_opens_is_told_at_once_and_all_gone_is_said_so() {
    let mut w = world("holds-before", &[("home/files", 5, 5)]);
    settle(&mut w.harness, "the banner", |h| h.count(BANNER) == 1);
    assert!(
        w.harness
            .html()
            .contains("All 5 files were removed on the server.")
    );
    assert_eq!(*w.sync.watched.lock().expect("watched"), 1);
}

#[test]
fn remove_confirms_the_discard_and_the_banner_goes_with_the_empty_signal() {
    let mut w = world("holds-remove", &[("home/files", 3, 4)]);
    settle(&mut w.harness, "the banner", |h| h.count(BANNER) == 1);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the call", |_| {
        !w.sync.confirmed().is_empty()
    });
    assert_eq!(w.sync.confirmed(), ["home/files"]);
    settle(&mut w.harness, "the banner to go", |h| h.count(BANNER) == 0);
}

#[test]
fn keep_calls_nothing() {
    let mut w = world("holds-keep", &[("home/files", 3, 4)]);
    settle(&mut w.harness, "the banner", |h| h.count(BANNER) == 1);
    click(&mut w.harness, KEEP);
    settle(&mut w.harness, "the banner to get out of the way", |h| {
        h.count(BANNER) == 0
    });
    assert!(w.sync.confirmed().is_empty(), "Keep calls nothing");
}

#[test]
fn a_hold_that_ended_unseen_is_refreshed_when_remove_finds_nothing_held() {
    let mut w = world("holds-stale", &[("home/files", 3, 4)]);
    settle(&mut w.harness, "the banner", |h| h.count(BANNER) == 1);
    w.sync.forget("home/files");
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the call", |_| {
        !w.sync.confirmed().is_empty()
    });
    settle(&mut w.harness, "the stale banner to go", |h| {
        h.count(BANNER) == 0
    });
}

#[test]
fn a_server_that_lists_its_files_again_ends_the_hold_by_itself() {
    let mut w = world("holds-heal", &[("home/files", 3, 4)]);
    settle(&mut w.harness, "the banner", |h| h.count(BANNER) == 1);
    w.rt.block_on(w.sync.heal(&w.sync_bus, "home/files"));
    settle(&mut w.harness, "the banner to go", |h| h.count(BANNER) == 0);
    assert!(w.sync.confirmed().is_empty());
}
