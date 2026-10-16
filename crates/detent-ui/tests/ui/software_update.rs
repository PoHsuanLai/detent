//! The Software Update page over a service that answers from memory: what each standing shows,
//! what a press asks of the service, and the page without a service.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, title_of};
use crate::support::updates::{Asked, NOW, Service};
use detent_model::{
    ByteCount, Permille, ProblemKind, UnixSeconds, UpdateOffer, UpdatePhase, UpdateStatus,
    UpdateVersion,
};
use detent_ui::Opening;
use ds_harness::{Driver, Harness, Query};
use std::time::Duration;

const ROW: &str = ".updates .ds-field-row";
const BUTTONS: &str = ".updates .ds-field-row .ds-button";
const CHECK: &str = "[aria-label=\"Check for Updates\"]";
const MORE_INFO: &str = "[aria-label=\"More Info\u{2026}\"]";
const UPDATE_NOW: &str = "[aria-label=\"Update Now\"]";
const TRY_AGAIN: &str = "[aria-label=\"Try Again\"]";
const LOG_OUT: &str = "[aria-label=\"Log Out\u{2026}\"]";
const NOTES: &str = ".updates-sheet";
const NOTICE: &str = ".updates-notice";
/// The note that says a newer version arrived, and what the service says (with the kind
/// `version-changed`) when its offer changed under an install.
const NEWER: &str = "A newer version arrived. Check what's new before installing.";
const OVERTAKEN: &str = "A different update is on offer now. Look at it again before installing.";
const BAR: &str = ".updates-meter .ds-progress";

fn software_update() -> Opening {
    Opening {
        page: Some(detent_model::PageSlug("software-update".to_owned())),
        query: String::new(),
    }
}

fn idle() -> UpdateStatus {
    UpdateStatus {
        phase: UpdatePhase::Idle,
        current: Some("0.3.1".to_owned()),
        offer: None,
        progress: Permille(0),
        last_checked: Some(UnixSeconds(NOW.0 - 300)),
        problem: None,
        problem_kind: ProblemKind::Other,
    }
}

fn version(text: &str) -> UpdateVersion {
    UpdateVersion::parse(text).expect("a version")
}

fn offered_version(phase: UpdatePhase, text: &str) -> UpdateStatus {
    UpdateStatus {
        phase,
        offer: Some(UpdateOffer {
            version: version(text),
            notes: "Faster start.\nFewer crashes.".to_owned(),
            size: ByteCount(45_600_000),
        }),
        ..idle()
    }
}

fn offered(phase: UpdatePhase) -> UpdateStatus {
    offered_version(phase, "0.4.0")
}

/// Lets the page hear what the service said: the follow loop runs on the harness's clock.
fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    for _ in 0..200 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

fn showing(harness: &mut Harness, what: &str, selector: &'static str) {
    settle(harness, what, |h| h.count(selector) > 0);
}

fn row_text(harness: &Harness) -> String {
    harness.text_of(ROW).unwrap_or_default()
}

fn world(name: &str, service: &Service) -> (Sandbox, Harness) {
    let sandbox = Sandbox::new(name).with_service(service.clone());
    let harness = sandbox.harness(software_update());
    (sandbox, harness)
}

#[test]
fn the_page_opens_from_its_slug_and_says_plainly_when_nothing_serves_updates() {
    let sandbox = Sandbox::new("swupdate-absent");
    let mut harness = sandbox.harness(software_update());
    showing(
        &mut harness,
        "the unavailable row",
        "[data-updates=\"unavailable\"]",
    );
    assert_eq!(page_of(&harness).as_deref(), Some("software-update"));
    assert_eq!(title_of(&harness).as_deref(), Some("Software Update"));
    let row = row_text(&harness);
    assert!(
        row.contains("Software updates are not available on this computer."),
        "{row}"
    );
    assert_eq!(harness.count(BUTTONS), 0, "nothing to press");
}

#[test]
fn an_up_to_date_computer_shows_its_version_and_when_it_last_checked() {
    let service = Service::answering(idle());
    let (_sandbox, mut harness) = world("swupdate-idle", &service);
    showing(&mut harness, "the version row", CHECK);
    let row = row_text(&harness);
    assert!(row.contains("Quire 0.3.1"), "{row}");
    assert!(row.contains("Your computer is up to date"), "{row}");
    assert_eq!(
        harness.text_of(".updates-note").as_deref(),
        Some("Last checked 5 minutes ago")
    );
    assert_eq!(service.asked(), [], "looking asks nothing");
}

#[test]
fn checking_asks_the_service_shows_a_spinner_and_then_what_it_found() {
    let service = Service::answering(idle());
    let (_sandbox, mut harness) = world("swupdate-check", &service);
    showing(&mut harness, "the button", CHECK);
    assert_eq!(harness.attr(CHECK, "aria-busy"), None, "not busy before");

    click(&mut harness, CHECK);
    assert_eq!(service.asked(), [Asked::Check]);
    settle(&mut harness, "the busy button", |h| {
        h.attr(CHECK, "aria-busy").as_deref() == Some("true")
    });
    assert!(row_text(&harness).contains("Checking for updates"));

    service.becomes(offered(UpdatePhase::Available));
    showing(&mut harness, "the offer", UPDATE_NOW);
    let row = row_text(&harness);
    assert!(row.contains("Quire 0.4.0"), "{row}");
    assert!(row.contains("45.6 MB"), "{row}");
    assert_eq!(
        harness.count(CHECK),
        0,
        "an offer replaces the check button"
    );
}

#[test]
fn more_info_shows_the_notes_and_done_puts_them_away() {
    let service = Service::answering(offered(UpdatePhase::Available));
    let (_sandbox, mut harness) = world("swupdate-notes", &service);
    showing(&mut harness, "More Info", MORE_INFO);
    assert_eq!(harness.count(NOTES), 0);

    click(&mut harness, MORE_INFO);
    showing(&mut harness, "the notes", NOTES);
    let notes = harness.text_of(NOTES).unwrap_or_default();
    assert!(notes.contains("Quire 0.4.0"), "{notes}");
    assert!(notes.contains("Faster start."), "{notes}");
    assert!(notes.contains("Fewer crashes."), "{notes}");
    assert_eq!(service.asked(), [], "reading notes asks nothing");

    click(&mut harness, ".updates-sheet .ds-button");
    settle(&mut harness, "the sheet leaving", |h| h.count(NOTES) == 0);
}

#[test]
fn update_now_downloads_with_a_bar_installs_without_one_and_ends_with_log_out() {
    let service = Service::answering(offered(UpdatePhase::Available));
    let (sandbox, mut harness) = world("swupdate-install", &service);
    showing(&mut harness, "Update Now", UPDATE_NOW);

    click(&mut harness, UPDATE_NOW);
    assert_eq!(service.asked(), [Asked::Install(version("0.4.0"))]);
    settle(&mut harness, "the download", |h| {
        row_text(h).contains("Downloading")
    });
    assert_eq!(
        harness.count(BUTTONS),
        0,
        "nothing to press while it downloads"
    );

    service.becomes(UpdateStatus {
        progress: Permille(250),
        ..offered(UpdatePhase::Downloading)
    });
    settle(&mut harness, "a quarter", |h| {
        h.attr(BAR, "aria-valuenow").as_deref() == Some("25")
    });
    assert_eq!(
        harness.text_of(".updates-meter").as_deref(),
        Some("11.4 MB of 45.6 MB")
    );

    service.becomes(offered(UpdatePhase::Installing));
    settle(&mut harness, "the pending bar", |h| {
        h.count(BAR) == 1 && h.attr(BAR, "aria-valuenow").is_none()
    });
    assert!(row_text(&harness).contains("Installing"));

    service.becomes(UpdateStatus {
        phase: UpdatePhase::Installed,
        current: Some("0.4.0".to_owned()),
        offer: None,
        ..idle()
    });
    showing(&mut harness, "Log Out", LOG_OUT);
    let row = row_text(&harness);
    assert!(row.contains("Quire 0.4.0"), "{row}");
    assert!(row.contains("Log out to finish the update"), "{row}");
    assert_eq!(sandbox.leaves().started(), 0, "nothing logs out by itself");

    click(&mut harness, LOG_OUT);
    assert_eq!(sandbox.leaves().started(), 1);
}

#[test]
fn a_failure_is_told_in_the_services_words_and_try_again_asks_again() {
    let failed = UpdateStatus {
        phase: UpdatePhase::Failed,
        problem: Some("There is no network.".to_owned()),
        ..offered(UpdatePhase::Failed)
    };
    let service = Service::answering(failed);
    let (_sandbox, mut harness) = world("swupdate-failed", &service);
    showing(&mut harness, "Try Again", TRY_AGAIN);
    assert!(row_text(&harness).contains("There is no network."));

    click(&mut harness, TRY_AGAIN);
    assert_eq!(
        service.asked(),
        [Asked::Install(version("0.4.0"))],
        "what was on offer is installed again"
    );
}

#[test]
fn a_refused_request_is_told_in_a_toast_and_changes_nothing() {
    let service = Service::answering(idle());
    let (_sandbox, mut harness) = world("swupdate-refused", &service);
    showing(&mut harness, "the button", CHECK);
    service.refuses("Another update is in progress.");

    click(&mut harness, CHECK);
    assert_eq!(service.asked(), [Asked::Check]);
    settle(&mut harness, "the toast", |h| h.count(".ds-toast") == 1);
    let toast = harness.text_of(".ds-toast").unwrap_or_default();
    assert!(toast.contains("Another update is in progress."), "{toast}");
    assert!(row_text(&harness).contains("Your computer is up to date"));
}

#[test]
fn a_service_that_goes_away_and_comes_back_is_followed() {
    let service = Service::answering(idle());
    let (_sandbox, mut harness) = world("swupdate-away", &service);
    showing(&mut harness, "the version row", CHECK);

    service.goes();
    showing(
        &mut harness,
        "the unavailable row",
        "[data-updates=\"unavailable\"]",
    );
    assert_eq!(harness.count(CHECK), 0);

    service.becomes(idle());
    showing(&mut harness, "the version row again", CHECK);
}

#[test]
fn an_offer_that_changed_after_it_was_shown_is_not_installed_and_the_new_one_is_shown() {
    let service = Service::answering(offered(UpdatePhase::Available));
    let (_sandbox, mut harness) = world("swupdate-changed", &service);
    showing(&mut harness, "Update Now", UPDATE_NOW);
    assert!(row_text(&harness).contains("Quire 0.4.0"));
    assert_eq!(harness.count(NOTICE), 0, "no note before anything changed");

    // 0.5.0 lands before the person presses; the page has not heard yet.
    service.changes_unseen(offered_version(UpdatePhase::Available, "0.5.0"));
    click(&mut harness, UPDATE_NOW);
    assert_eq!(
        service.asked(),
        [Asked::Install(version("0.4.0"))],
        "what was shown is what was named"
    );
    showing(&mut harness, "the note", NOTICE);
    assert_eq!(harness.text_of(NOTICE).as_deref(), Some(NEWER));
    let row = row_text(&harness);
    assert!(row.contains("Quire 0.5.0"), "{row}");
    assert_eq!(harness.count(".ds-toast"), 0, "a note, not an error");
    assert_eq!(harness.count(BAR), 0, "nothing started");

    // Pressing again names the version now shown, and the note goes.
    click(&mut harness, UPDATE_NOW);
    assert_eq!(
        service.asked(),
        [
            Asked::Install(version("0.4.0")),
            Asked::Install(version("0.5.0"))
        ]
    );
    settle(&mut harness, "the download", |h| {
        row_text(h).contains("Downloading")
    });
    assert_eq!(harness.count(NOTICE), 0);
}

#[test]
fn a_failure_that_says_the_offer_changed_shows_the_new_version_with_the_note() {
    let overtaken = UpdateStatus {
        problem: Some(OVERTAKEN.to_owned()),
        problem_kind: ProblemKind::VersionChanged,
        ..offered_version(UpdatePhase::Failed, "0.5.0")
    };
    let service = Service::answering(overtaken);
    let (_sandbox, mut harness) = world("swupdate-overtaken", &service);
    showing(&mut harness, "Update Now", UPDATE_NOW);
    assert_eq!(harness.text_of(NOTICE).as_deref(), Some(NEWER));
    let row = row_text(&harness);
    assert!(row.contains("Quire 0.5.0"), "{row}");
    assert!(!row.contains(OVERTAKEN), "{row}");
    assert_eq!(harness.count(TRY_AGAIN), 0);
    assert_eq!(service.asked(), [], "looking asks nothing");
}

#[test]
fn nothing_on_offer_shows_no_way_to_install_and_asks_nothing() {
    let service = Service::answering(idle());
    let (_sandbox, mut harness) = world("swupdate-nothing", &service);
    showing(&mut harness, "the version row", CHECK);
    assert_eq!(harness.count(UPDATE_NOW), 0);
    assert_eq!(harness.count(TRY_AGAIN), 0);
    assert_eq!(service.asked(), []);
}

#[test]
fn a_refusal_to_install_is_told_in_a_toast_in_the_services_words() {
    let service = Service::answering(offered(UpdatePhase::Available));
    let (_sandbox, mut harness) = world("swupdate-install-refused", &service);
    showing(&mut harness, "Update Now", UPDATE_NOW);
    service.refuses("An update is already in progress.");

    click(&mut harness, UPDATE_NOW);
    settle(&mut harness, "the toast", |h| h.count(".ds-toast") == 1);
    let toast = harness.text_of(".ds-toast").unwrap_or_default();
    assert!(
        toast.contains("An update is already in progress."),
        "{toast}"
    );
    assert_eq!(harness.count(NOTICE), 0, "a refusal is not a changed offer");
}
