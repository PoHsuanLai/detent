//! A deletion the sync service holds for the person's word, on the account's sync row: over a fake
//! accountd and a fake syncd on a private bus.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held};
use crate::support::bus::PrivateBus;
use crate::support::drive::click;
use crate::support::handover::HandsOver;
use crate::support::syncd::Syncd;
use detent_callers::{RoleCheck, introduce_service, private_peers, roles};
use detent_client::Vouch;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Harness;
use ds_settings::live::{AnyOwner, Verdict};
use ferry_bus::SYNC_BUS;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const HOME: &str = "#account-home";
const BANNER: &str = ".accounts-hold .ds-inline-banner";
const KEEP: &str = ".accounts-hold [data-answers=\"return\"]";
const REMOVE: &str = ".accounts-hold [data-role=\"destructive\"]";
const KEEP_FILES: &str = "Keep this account's files on this computer";

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

struct World {
    harness: Harness,
    sync: Syncd,
    /// The program that takes the sync name from `sync` when `handover` fires.
    successor: Syncd,
    handover: Arc<AtomicBool>,
    _successor_bus: zbus::Connection,
    sync_bus: zbus::Connection,
    rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
    _accounts: zbus::Connection,
    _served: ds_settings::live::Served,
    _fake: Accountd,
}

/// Who the window takes the services' programs to be.
enum Trust {
    /// Every one of them is the service it plays.
    Everyone,
    /// Only accountd is; the program that owns the sync service's name is nobody the table knows.
    AccountsOnly,
    /// Every one is, and once `World::handover` fires the next time the sync service is vouched
    /// for, the name passes from it to the successor right after the answer is given.
    HandsOver,
}

/// The Accounts page over the fakes, with `before` holds already on when the window opens.
fn world(name: &str, before: &[(&str, u64, u64)]) -> World {
    world_trusting(name, before, Trust::Everyone)
}

/// The same, the window taking the services for what `trust` says.
fn world_trusting(name: &str, before: &[(&str, u64, u64)], trust: Trust) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemons");
    let bus = PrivateBus::start(name);
    let fake = Accountd::new(
        Verdict::Allow,
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
    let successor = Syncd::default();
    let successor_bus = rt.block_on(successor.serve_unnamed(&bus));
    let mut handover = Arc::new(AtomicBool::new(false));
    let vouch = match trust {
        Trust::Everyone => Vouch::of(AnyOwner),
        Trust::HandsOver => {
            let (check, armed) = HandsOver::new(SYNC_BUS, sync_bus.clone(), successor_bus.clone());
            handover = armed;
            Vouch::of(check)
        }
        Trust::AccountsOnly => {
            let watcher = rt.block_on(bus.connect());
            let peers = private_peers(&watcher);
            let accountd = accounts.unique_name().expect("a unique name").to_string();
            introduce_service(&peers, &accountd, &roles::ACCOUNTS);
            Vouch::of(RoleCheck::new(peers))
        }
    };
    let mut harness = Sandbox::new(name)
        .on_bus_vouched(&bus.address, vouch)
        .harness(opening);
    settle(&mut harness, "the list", |h| h.count(".accounts-list") == 1);
    click(&mut harness, HOME);
    settle(&mut harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
    World {
        harness,
        sync,
        successor,
        handover,
        _successor_bus: successor_bus,
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
fn a_program_that_took_the_sync_name_is_not_listened_to_and_is_never_told_to_discard() {
    let mut w = world_trusting(
        "holds-squatter",
        &[("home/files", 5, 5)],
        Trust::AccountsOnly,
    );
    // Time enough for a banner to show if the squatter's hold were believed.
    for _ in 0..60 {
        w.harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(w.harness.count(BANNER), 0, "its hold is not shown");
    assert_eq!(
        *w.sync.watched.lock().expect("watched"),
        0,
        "it was not joined"
    );
    assert!(w.sync.confirmed().is_empty(), "nothing was confirmed to it");
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

#[test]
fn a_discard_goes_to_the_program_that_was_checked_even_when_the_name_changes_hands_after() {
    let mut w = world_trusting("holds-handover", &[("home/files", 3, 4)], Trust::HandsOver);
    settle(&mut w.harness, "the banner", |h| h.count(BANNER) == 1);
    w.handover.store(true, Ordering::SeqCst);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the call", |_| {
        !w.sync.confirmed().is_empty()
    });
    assert_eq!(
        w.sync.confirmed(),
        ["home/files"],
        "the program checked took it"
    );
    assert!(
        w.successor.confirmed().is_empty(),
        "the program that took the name after the check was sent nothing"
    );
}
