//! The page when inferd is missing, refusing or starting late.

use super::*;
use detent_model::Absence;
use detent_system::bus::Daemon;
use detent_system::modules::FakeModules;

#[test]
fn an_absent_inferd_blocks_nothing_on_the_page() {
    let sandbox = Sandbox::new("intelligence-absent-writes");
    let mut harness = sandbox.harness(intelligence());
    assert_eq!(sandbox.read("inferd/inferd.toml"), None);
    let local_only = "[aria-label=\"Keep AI on this computer\"] .ds-toggle";
    let before = harness.attr(local_only, "aria-checked");
    assert_eq!(before.as_deref(), Some("true"), "local-only starts on");

    click(&mut harness, local_only);
    assert_eq!(
        harness.attr(local_only, "aria-checked").as_deref(),
        Some("false")
    );
    let written = sandbox
        .read("inferd/inferd.toml")
        .expect("the file row wrote");
    assert!(written.contains("local_only = \"off\""), "{written}");
}

#[test]
fn a_bus_with_no_inferd_on_it_says_inferd_is_not_running() {
    let sandbox = Sandbox::new("intelligence-no-inferd").with_modules(FakeModules::new());
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the picker giving up", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("Models are not available now"));
}

#[test]
fn inferd_refusing_settings_is_said_in_its_own_words() {
    let sandbox = Sandbox::new("intelligence-refused");
    let fake = Inferd::new(Policy::Refuse);
    let mut harness = sandbox.with_modules(fake.hosted()).harness(intelligence());
    settle(&mut harness, "the refusal", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("Open Settings from the desktop"));
}

#[test]
fn an_inferd_that_starts_after_the_page_opened_is_found_and_one_that_stops_is_missed() {
    let sandbox = Sandbox::new("intelligence-live");
    let modules = FakeModules::new();
    let mut harness = sandbox
        .with_modules(modules.clone())
        .harness(intelligence());
    settle(&mut harness, "the card", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("Models are not available now"));
    assert_eq!(harness.count(".picker-kind"), 0);

    let fake = Inferd::new(Policy::Allow);
    fake.host_on(&modules);
    settle(&mut harness, "the picker to appear", |h| {
        h.count(".picker-kind") == 2
    });
    assert_eq!(harness.count(".picker-kind .ds-field-row"), 4);

    modules.away(&Daemon::inferd(), Absence::NotRunning);
    settle(&mut harness, "the card to return", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("Models are not available now"));
    assert_eq!(harness.count(".picker-kind"), 0);

    fake.host_on(&modules);
    settle(&mut harness, "the picker to come back", |h| {
        h.count(".picker-kind") == 2
    });
}
