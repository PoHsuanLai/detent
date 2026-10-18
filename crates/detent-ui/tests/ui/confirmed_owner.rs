//! A call goes to the program that was checked, not to whoever holds the service's name when the
//! call is made: the Spaces page over a fake registry, and the Privacy page over a fake portal,
//! that lose the name to a second one the moment the window has asked who owns it.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::drive::click;
use crate::support::handover::HandsOver;
use crate::support::portal;
use crate::support::spaces::{Fakes, serve, serve_unnamed};
use detent_model::PageSlug;
use detent_ui::{Opening, Vouch};
use ds_harness::Harness;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;

const ACCOUNTS: &str = "org.quire.Accounts1";
const TRIP: &str = "[aria-label=\"Remove Trip\"]";
const IMPOSTER: &str = "[aria-label=\"Remove Imposter\"]";
const SHEET: &str = ".space-sheet";
const REMOVE: &str = ".space-sheet .ds-button[data-answers=\"return\"]";

fn spaces() -> Opening {
    Opening {
        page: Some(PageSlug("spaces".to_owned())),
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

/// The registry that lists Trip, a second registry that lists Imposter and takes accountd's name
/// from the first once `handover` is armed and the window has asked who owns it.
struct World {
    original: Fakes,
    successor: Fakes,
    handover: Arc<AtomicBool>,
    sandbox: Sandbox,
    rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
}

fn world(name: &str) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fakes");
    let bus = PrivateBus::start(name);
    let original = rt.block_on(serve(&bus, &[("space-1", "Trip")], true));
    let successor = rt.block_on(serve_unnamed(&bus, &[("space-9", "Imposter")]));
    let (check, handover) = HandsOver::new(
        ACCOUNTS,
        original.registry.clone(),
        successor.registry.clone(),
    );
    let sandbox = Sandbox::new(name).on_bus_vouched(&bus.address, Vouch::of(check));
    World {
        original,
        successor,
        handover,
        sandbox,
        rt,
        _bus: bus,
    }
}

/// The unique name that owns `name` now.
fn owner(rt: &tokio::runtime::Runtime, via: &zbus::Connection, name: &str) -> String {
    rt.block_on(async {
        let bus = DBusProxy::new(via).await.expect("the bus");
        let name = BusName::try_from(name).expect("a name");
        bus.get_name_owner(name)
            .await
            .expect("an owner")
            .to_string()
    })
}

fn unique(fakes: &Fakes) -> String {
    fakes
        .registry
        .unique_name()
        .expect("a unique name")
        .to_string()
}

fn log_of(fakes: &Fakes) -> Vec<String> {
    fakes.log.lock().expect("log").clone()
}

#[test]
fn the_list_is_read_from_the_program_that_was_checked() {
    let w = world("owner-list");
    assert_eq!(
        owner(&w.rt, &w.original.registry, ACCOUNTS),
        unique(&w.original),
        "the control: it owns the name"
    );
    w.handover.store(true, Ordering::SeqCst);
    let mut harness = w.sandbox.harness(spaces());
    settle(&mut harness, "the list", |h| h.count(TRIP) == 1);
    assert_eq!(
        harness.count(IMPOSTER),
        0,
        "the program that took the name is not listed"
    );
    assert_eq!(
        owner(&w.rt, &w.original.registry, ACCOUNTS),
        unique(&w.successor),
        "the name did change hands after the check"
    );
}

#[test]
fn a_removal_goes_to_the_program_that_was_checked() {
    let w = world("owner-remove");
    let mut harness = w.sandbox.harness(spaces());
    settle(&mut harness, "the list", |h| h.count(TRIP) == 1);
    click(&mut harness, TRIP);
    settle(&mut harness, "the sheet", |h| h.count(SHEET) == 1);
    w.handover.store(true, Ordering::SeqCst);
    click(&mut harness, REMOVE);
    settle(&mut harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(
        owner(&w.rt, &w.original.registry, ACCOUNTS),
        unique(&w.successor),
        "the name did change hands after the check"
    );
    assert_eq!(
        log_of(&w.original),
        [
            r#"memory space-1 {"memories":"move_to_apps","history":"keep"}"#.to_owned(),
            "registry remove space-1".to_owned(),
        ],
        "the program checked was asked"
    );
    assert!(
        log_of(&w.successor).is_empty(),
        "the program that took the name after the check was asked nothing: {:?}",
        log_of(&w.successor)
    );
}

#[test]
fn an_app_is_taken_back_at_the_portal_that_was_checked() {
    const MEET: &str = "org.example.Meet";
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fakes");
    let bus = PrivateBus::start("owner-portal");
    let apps = [portal::Remembered {
        app: MEET,
        label: "Meet",
        mode: 2,
        granted: 1,
    }];
    let original = rt.block_on(portal::serve(&bus, &apps));
    let successor = rt.block_on(portal::serve_unnamed(&bus, &apps));
    let (check, handover) = HandsOver::new(
        portal::NAME,
        original.connection.clone(),
        successor.connection.clone(),
    );
    let sandbox = Sandbox::new("owner-portal").on_bus_vouched(&bus.address, Vouch::of(check));
    sandbox.write_application(
        MEET,
        "[Desktop Entry]\nType=Application\nName=Video Meet\nExec=flatpak run org.example.Meet\nX-Flatpak=org.example.Meet\n",
    );
    let mut harness = sandbox.harness(Opening {
        page: Some(PageSlug("privacy".to_owned())),
        query: String::new(),
    });
    let remove = "[aria-label=\"Remove Video Meet\"]";
    settle(&mut harness, "the list", |h| h.count(remove) == 1);
    handover.store(true, Ordering::SeqCst);
    click(&mut harness, remove);
    settle(&mut harness, "the removal", |_| {
        !original.revoked().is_empty()
    });
    assert_eq!(
        owner(&rt, &original.connection, portal::NAME),
        successor
            .connection
            .unique_name()
            .expect("a unique name")
            .to_string(),
        "the name did change hands after the check"
    );
    assert_eq!(original.revoked(), [MEET], "the program checked was asked");
    assert!(
        successor.revoked().is_empty(),
        "the program that took the name after the check was asked nothing"
    );
}
