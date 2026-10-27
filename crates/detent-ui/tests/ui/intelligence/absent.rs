//! The page when inferd is missing, refusing or starting late.

use super::*;

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
    let sandbox = Sandbox::new("intelligence-no-inferd");
    let rt = runtime();
    let bus = PrivateBus::start("no-inferd");
    let sandbox = sandbox.on_bus(&bus.address);
    let mut harness = sandbox.harness(intelligence());
    settle(&mut harness, "the picker giving up", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("unavailable")
    });
    assert!(harness.html().contains("Models are not available now"));
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
    assert!(harness.html().contains("Open Settings from the desktop"));
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
    assert!(harness.html().contains("Models are not available now"));
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
    assert!(harness.html().contains("Models are not available now"));
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
            .any(|n| n.as_str() == porter_dbus::names::INFERENCE_BUS),
        "nothing owns inferd's name: the page only looked"
    );
}
