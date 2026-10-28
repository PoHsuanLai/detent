//! Showing a page asks the bus to start the daemons that page reads, and only those: a bus with a
//! service file for each daemon records which were started, and a name with no service file stays
//! absent, so the page says it is unavailable as before.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Harness;
use porter_dbus::accounts::ACCOUNTS_BUS;
use porter_dbus::inference::INFERENCE_BUS;
use porter_dbus::sync::SYNC_BUS;
use std::time::Duration;

/// A daemon no page reads: starting it would be a start for a page nobody opened.
const MEMORY_BUS: &str = "org.quire.Memory1";

const ALL: [&str; 4] = [ACCOUNTS_BUS, INFERENCE_BUS, SYNC_BUS, MEMORY_BUS];

fn page(slug: &str) -> Opening {
    Opening {
        page: Some(PageSlug(slug.to_owned())),
        query: String::new(),
    }
}

/// Shows `slug` on a bus that can start `services`, lets the window run for a while and returns the
/// names the bus started, with the window.
fn visit(test: &str, slug: &str, services: &[&str]) -> (Vec<String>, Harness) {
    let bus = PrivateBus::start_with_services(test, services);
    let mut harness = Sandbox::new(test).on_bus(&bus.address).harness(page(slug));
    // The starts are asked of the bus on real threads; the budget is generous and a page that
    // starts nothing waits it out.
    for _ in 0..100 {
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    (bus.started(), harness)
}

#[test]
fn the_accounts_page_starts_the_three_daemons_it_reads() {
    let (started, _) = visit("activate-accounts", "accounts", &ALL);
    assert_eq!(started, [ACCOUNTS_BUS, INFERENCE_BUS, SYNC_BUS]);
}

#[test]
fn the_intelligence_page_starts_the_daemons_it_reads_and_not_memory() {
    let (started, _) = visit("activate-intelligence", "intelligence", &ALL);
    assert_eq!(started, [ACCOUNTS_BUS, INFERENCE_BUS, SYNC_BUS]);
}

#[test]
fn the_spaces_page_starts_only_the_registry() {
    let (started, _) = visit("activate-spaces", "spaces", &ALL);
    assert_eq!(started, [ACCOUNTS_BUS]);
}

#[test]
fn a_page_that_reads_files_only_starts_nothing() {
    let (started, _) = visit("activate-appearance", "appearance", &ALL);
    assert_eq!(started, Vec::<String>::new());
}

#[test]
fn a_name_without_a_service_file_stays_absent_and_the_page_says_so() {
    let (started, harness) = visit("activate-unknown", "accounts", &[INFERENCE_BUS]);
    assert_eq!(
        started,
        [INFERENCE_BUS],
        "only the name that can be started"
    );
    assert_eq!(
        harness.attr(".accounts", "data-accounts").as_deref(),
        Some("unavailable"),
        "{}",
        harness.html()
    );
    assert!(
        harness
            .html()
            .contains("The accounts service is not running"),
        "no daemon took its name, so the pane says none is running"
    );
}
