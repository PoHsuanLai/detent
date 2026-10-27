//! The Privacy page's screen-sharing apps over a fake portal on a private bus: the apps it
//! remembers are listed by the id it proved them by, Remove takes one back and the row goes, a
//! portal that is not running is started on demand and listed once the program that took its name
//! holds the portal role, and a portal that cannot be started or is not the portal leaves the rest
//! of the page as it was.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::compositor::{Compositor, Row};
use crate::support::drive::click;
use crate::support::portal::{Fake, NAME, Remembered, serve, serve_unnamed};
use crate::support::updates::NOW;
use detent_callers::{RoleCheck, introduce_service, private_peers, roles};
use detent_client::Vouch;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Harness;
use ds_settings::live::AnyOwner;
use std::time::Duration;

const MEET: &str = "org.example.Meet";
const CALL: &str = "org.example.Call";
const GRABBER: &str = "org.evil.Grabber";
const HOUR: u64 = 3600;
const DAY: u64 = 24 * HOUR;

const GROUP: &str = "[data-screen-sharing=\"list\"]";
const UNAVAILABLE: &str = "[data-screen-sharing=\"unavailable\"]";
const REMOVE_MEET: &str = "[aria-label=\"Remove Video Meet\"]";

fn privacy() -> Opening {
    Opening {
        page: Some(PageSlug("privacy".to_owned())),
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

/// The apps the portal remembers: one a system desktop entry names, one that has no entry, and
/// one whose entry the portal reads as "System Settings".
fn remembered() -> Vec<Remembered> {
    let ago = |seconds: u64| NOW.0.unsigned_abs() - seconds;
    vec![
        Remembered {
            app: MEET,
            label: "Meet",
            mode: 2,
            granted: ago(3 * DAY),
        },
        Remembered {
            app: CALL,
            label: "Call",
            mode: 1,
            granted: ago(2 * HOUR),
        },
        Remembered {
            app: GRABBER,
            label: "System Settings",
            mode: 2,
            granted: ago(DAY),
        },
    ]
}

/// Who the window takes the portal's program to be.
enum Trust {
    /// Every program is the service it plays.
    Anyone,
    /// The portal's program holds the portal role in the desktop's table.
    Introduced,
    /// Nobody is known to the desktop's table.
    Nobody,
}

/// Whether the portal is there, and how.
enum Presence {
    /// No portal, and the bus has no way to start one.
    Unstartable,
    /// A portal holding these apps, already on the bus.
    Running(Vec<Remembered>),
    /// A portal holding these apps that takes its name only when the bus starts it.
    OnDemand(Vec<Remembered>),
}

struct World {
    portal: Option<Fake>,
    harness: Harness,
    rt: tokio::runtime::Runtime,
    bus: PrivateBus,
    _compositor: (zbus::Connection, ds_settings::live::Served),
}

/// The Privacy page over a compositor listing one program, and the portal as `presence` says.
fn world(name: &str, presence: Presence, trust: Trust) -> World {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fakes");
    let bus = match presence {
        Presence::OnDemand(_) => PrivateBus::start_with_lingering_services(name, &[NAME]),
        Presence::Unstartable | Presence::Running(_) => PrivateBus::start(name),
    };
    let compositor = Compositor::holding(vec![Row::person("screen", "aaa", "/usr/bin/grim", "on")]);
    let served = rt.block_on(compositor.serve_on(&bus));
    let portal = match &presence {
        Presence::Unstartable => None,
        Presence::Running(apps) => Some(rt.block_on(serve(&bus, apps))),
        Presence::OnDemand(apps) => Some(rt.block_on(serve_unnamed(&bus, apps))),
    };
    let vouch = match (trust, &portal) {
        (Trust::Anyone, _) => Vouch::of(AnyOwner),
        (Trust::Introduced, Some(portal)) => {
            let peers = private_peers(&rt.block_on(bus.connect()));
            let unique = portal.connection.unique_name().expect("a unique name");
            introduce_service(&peers, unique.as_str(), &roles::PORTAL);
            Vouch::of(RoleCheck::new(peers))
        }
        (Trust::Introduced | Trust::Nobody, _) => {
            Vouch::of(RoleCheck::new(private_peers(&rt.block_on(bus.connect()))))
        }
    };
    let sandbox = Sandbox::new(name).on_bus_vouched(&bus.address, vouch);
    sandbox.write_application(
        MEET,
        "[Desktop Entry]\nType=Application\nName=Video Meet\nExec=flatpak run org.example.Meet\nX-Flatpak=org.example.Meet\n",
    );
    let harness = sandbox.harness(privacy());
    World {
        portal,
        harness,
        rt,
        bus,
        _compositor: served,
    }
}

/// `html` without its `<style>` blocks: the stylesheet's comments name System Settings too.
fn without_styles(html: &str) -> String {
    let mut rest = html;
    let mut kept = String::with_capacity(html.len());
    while let Some(start) = rest.find("<style") {
        kept.push_str(&rest[..start]);
        rest = rest[start..]
            .find("</style>")
            .map_or("", |end| &rest[start + end + "</style>".len()..]);
    }
    kept.push_str(rest);
    kept
}

fn listed(w: &mut World) {
    settle(&mut w.harness, "the list", |h| h.count(GROUP) == 1);
}

/// The programs above are on the page too. They load on their own and push the rows below down
/// when they come, so a click aimed before then can land on another row.
fn steady(w: &mut World) {
    listed(w);
    settle(&mut w.harness, "the programs above", |h| {
        h.count("[data-privacy=\"list\"]") == 1
    });
}

#[test]
fn the_apps_the_portal_remembers_are_named_by_the_id_it_proved_and_say_when_and_for_how_long() {
    let mut w = world(
        "screen-sharing-list",
        Presence::Running(remembered()),
        Trust::Anyone,
    );
    listed(&mut w);
    let html = w.harness.html();
    for text in [
        "Screen sharing",
        "Video Meet",
        "Allowed 3 days ago, until you remove it",
        CALL,
        "Allowed 2 hours ago, while it is open",
        GRABBER,
        "Allowed yesterday, until you remove it",
    ] {
        assert!(html.contains(text), "{text} in {html}");
    }
    assert!(
        !without_styles(&html).contains("System Settings"),
        "the portal's own name for an app is not shown: {html}"
    );
    let at = |text: &str| html.find(text).unwrap_or(usize::MAX);
    assert!(
        at(CALL) < at(GRABBER) && at(GRABBER) < at("Video Meet"),
        "the app allowed last comes first"
    );
    assert_eq!(w.harness.count("[aria-label^=\"Remove \"]"), 3);
    // The programs above load on their own; they come too.
    settle(&mut w.harness, "the programs above", |h| {
        h.count("[data-privacy=\"list\"]") == 1
    });
}

#[test]
fn remove_takes_the_app_back_and_its_row_goes() {
    let mut w = world(
        "screen-sharing-remove",
        Presence::Running(remembered()),
        Trust::Anyone,
    );
    steady(&mut w);
    let portal = w.portal.as_ref().expect("a portal");
    assert!(portal.revoked().is_empty());
    click(&mut w.harness, REMOVE_MEET);
    settle(&mut w.harness, "the row to go", |h| {
        h.count(REMOVE_MEET) == 0
    });
    let portal = w.portal.as_ref().expect("a portal");
    assert_eq!(portal.revoked(), [MEET]);
    assert_eq!(
        w.harness.count("[aria-label^=\"Remove \"]"),
        2,
        "the other two stay"
    );
    assert!(w.harness.html().contains(CALL));
}

#[test]
fn a_refusal_is_told_in_plain_words_and_the_row_stays() {
    let mut w = world(
        "screen-sharing-refused",
        Presence::Running(remembered()),
        Trust::Anyone,
    );
    steady(&mut w);
    w.portal.as_ref().expect("a portal").refuse_next();
    click(&mut w.harness, REMOVE_MEET);
    settle(&mut w.harness, "the toast", |h| h.count(".ds-toast") == 1);
    let toast = w.harness.text_of(".ds-toast").unwrap_or_default();
    assert!(
        toast.contains("Screen sharing could not be changed"),
        "{toast}"
    );
    assert!(w.portal.as_ref().expect("a portal").revoked().is_empty());
    assert_eq!(w.harness.count(REMOVE_MEET), 1, "the app is still listed");
    // A second press works: the first refusal did not leave the list stuck. The toast sits over
    // the last row at this window size, so the press waits for it to go.
    settle(&mut w.harness, "the toast to go", |h| {
        h.count(".ds-toast") == 0
    });
    click(&mut w.harness, REMOVE_MEET);
    settle(&mut w.harness, "the row to go", |h| {
        h.count(REMOVE_MEET) == 0
    });
}

#[test]
fn with_no_app_remembered_the_group_says_so() {
    let mut w = world(
        "screen-sharing-empty",
        Presence::Running(vec![]),
        Trust::Anyone,
    );
    listed(&mut w);
    let words = w.harness.text_of(GROUP).unwrap_or_default();
    assert!(
        words.contains("No apps can share your screen without asking"),
        "{words}"
    );
    assert_eq!(w.harness.count("[aria-label^=\"Remove \"]"), 0);
}

#[test]
fn a_portal_that_cannot_be_started_is_told_in_plain_words_and_the_rest_of_the_page_is_there() {
    let mut w = world(
        "screen-sharing-absent",
        Presence::Unstartable,
        Trust::Anyone,
    );
    settle(&mut w.harness, "the line", |h| h.count(UNAVAILABLE) == 1);
    let words = w.harness.text_of(UNAVAILABLE).unwrap_or_default();
    assert!(
        words.contains("The screen sharing service could not be started"),
        "{words}"
    );
    assert!(!words.contains("not running"), "{words}");
    for banned in ["portal", "D-Bus", "bus"] {
        assert!(!words.contains(banned), "{banned} in {words}");
    }
    assert_eq!(w.bus.started(), Vec::<String>::new());
    settle(&mut w.harness, "the programs", |h| {
        h.count("[data-privacy=\"list\"]") == 1
    });
}

#[test]
fn a_portal_that_is_not_running_is_started_and_the_apps_it_remembers_are_listed() {
    let mut w = world(
        "screen-sharing-on-demand",
        Presence::OnDemand(remembered()),
        Trust::Introduced,
    );
    // The bus starts the portal when the group is shown; the portal takes its name then.
    settle(&mut w.harness, "the bus to start the portal", |_| {
        w.bus.started() == [NAME]
    });
    let portal = w.portal.as_ref().expect("a portal");
    assert_eq!(portal.listed(), 0, "nothing is asked before it is up");
    w.rt.block_on(portal.connection.request_name(NAME))
        .expect("the portal's name");
    listed(&mut w);
    let portal = w.portal.as_ref().expect("a portal");
    assert_eq!(portal.listed(), 1);
    assert_eq!(w.harness.count("[aria-label^=\"Remove \"]"), 3);
    assert!(w.harness.html().contains("Video Meet"));
}

#[test]
fn a_portal_that_is_started_but_is_not_the_portal_is_not_asked_anything() {
    let mut w = world(
        "screen-sharing-on-demand-squatter",
        Presence::OnDemand(remembered()),
        Trust::Nobody,
    );
    settle(&mut w.harness, "the bus to start the portal", |_| {
        w.bus.started() == [NAME]
    });
    let portal = w.portal.as_ref().expect("a portal");
    w.rt.block_on(portal.connection.request_name(NAME))
        .expect("the portal's name");
    settle(&mut w.harness, "the line", |h| h.count(UNAVAILABLE) == 1);
    let words = w.harness.text_of(UNAVAILABLE).unwrap_or_default();
    assert!(words.contains("could not be confirmed"), "{words}");
    let portal = w.portal.as_ref().expect("a portal");
    assert_eq!(portal.listed(), 0, "it was not asked for its list");
}

#[test]
fn a_program_the_desktop_does_not_know_as_the_portal_is_not_asked_anything() {
    let mut w = world(
        "screen-sharing-squatter",
        Presence::Running(remembered()),
        Trust::Nobody,
    );
    settle(&mut w.harness, "the line", |h| h.count(UNAVAILABLE) == 1);
    let words = w.harness.text_of(UNAVAILABLE).unwrap_or_default();
    assert!(words.contains("could not be confirmed"), "{words}");
    let portal = w.portal.as_ref().expect("a portal");
    assert_eq!(portal.listed(), 0, "it was not asked for its list");
    assert_eq!(w.harness.count("[aria-label^=\"Remove \"]"), 0);
}

#[test]
fn the_program_that_holds_the_portal_role_is_listed() {
    let mut w = world(
        "screen-sharing-role",
        Presence::Running(remembered()),
        Trust::Introduced,
    );
    listed(&mut w);
    let portal = w.portal.as_ref().expect("a portal");
    assert_eq!(portal.listed(), 1);
    assert_eq!(w.harness.count("[aria-label^=\"Remove \"]"), 3);
}
