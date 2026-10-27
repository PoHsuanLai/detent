//! The page without its modules, a module that refuses, and a program that is not the module (the
//! last two over a private bus: whose name it is, is the bus's to say).

use super::*;

#[test]
fn no_bus_says_so_and_blocks_nothing_else_on_the_window() {
    let sandbox = Sandbox::new("accounts-no-bus");
    let harness = sandbox.harness(accounts());
    assert_eq!(
        harness.attr(".accounts", "data-accounts").as_deref(),
        Some("unavailable")
    );
    assert!(harness.html().contains("Unavailable"));
}

#[test]
fn a_bus_with_none_of_the_three_says_the_service_is_not_running() {
    let mut harness = Sandbox::new("accounts-none")
        .with_modules(FakeModules::new())
        .harness(accounts());
    settle(&mut harness, "the pane giving up", |h| {
        h.html().contains("The accounts service is not running")
    });
}

#[test]
fn accountd_refusing_settings_is_said_in_its_own_words() {
    let fake = Accountd::new(Policy::Refuse, identities());
    let mut harness = Sandbox::new("accounts-refused")
        .with_modules(fake.hosted())
        .harness(accounts());
    settle(&mut harness, "the refusal", |h| {
        h.html().contains("opened from the desktop")
    });
    assert_no_developer_words(&harness);
}

/// Which program owns accountd's name, as the desktop's caller table would see it.
enum Owner {
    /// A program that holds this role (accountd holds `accounts`).
    Holding(RoleName),
    /// A program in no row of the table: one that merely took the name.
    Nobody,
}

/// The window's owner check over peers that know `owner` (the connection that serves the module)
/// as `who` says. The peers' only answers are the ones this test gives; the real caller table and
/// the real unit manager are never asked.
fn vouching(
    rt: &tokio::runtime::Runtime,
    bus: &PrivateBus,
    owner: &zbus::Connection,
    who: Owner,
) -> Vouch {
    let watcher = rt.block_on(bus.connect());
    let peers = private_peers(&watcher);
    if let Owner::Holding(role) = who {
        let name = owner.unique_name().expect("a unique name").to_string();
        introduce_service(&peers, &name, &role);
    }
    Vouch::of(RoleCheck::new(peers))
}

#[test]
fn a_program_that_took_accountds_name_is_not_read_and_is_sent_nothing() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-squatter");
    let squatter = Accountd::new(Policy::Allow, identities());
    let daemon = rt.block_on(squatter.serve_on(&bus));
    let vouch = vouching(&rt, &bus, &daemon, Owner::Nobody);
    let mut harness = Sandbox::new("accounts-squatter")
        .on_bus_vouched(&bus.address, vouch)
        .harness(accounts());
    settle(&mut harness, "the pane giving up", |h| {
        h.html().contains("could not be confirmed")
    });
    assert_eq!(
        harness.count(".account-row-body"),
        0,
        "none of its rows are shown"
    );
    assert_eq!(squatter.calls(), 0, "it was not asked for anything");
    assert!(squatter.sets().is_empty());
    assert_no_developer_words(&harness);
}

#[test]
fn a_program_with_another_services_role_is_not_accountd_either() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-wrong-role");
    let other = Accountd::new(Policy::Allow, identities());
    let daemon = rt.block_on(other.serve_on(&bus));
    let vouch = vouching(&rt, &bus, &daemon, Owner::Holding(roles::SYNC));
    let mut harness = Sandbox::new("accounts-wrong-role")
        .on_bus_vouched(&bus.address, vouch)
        .harness(accounts());
    settle(&mut harness, "the pane giving up", |h| {
        h.html().contains("could not be confirmed")
    });
    assert_eq!(other.calls(), 0);
}

#[test]
fn the_program_that_holds_accountds_role_is_read() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-owner-vouched");
    let accountd = Accountd::new(Policy::Allow, identities());
    let daemon = rt.block_on(accountd.serve_on(&bus));
    let vouch = vouching(&rt, &bus, &daemon, Owner::Holding(roles::ACCOUNTS));
    let mut harness = Sandbox::new("accounts-owner-vouched")
        .on_bus_vouched(&bus.address, vouch)
        .harness(accounts());
    settle(&mut harness, "the rows", |h| {
        h.count(".account-row-body") == 2
    });
    assert!(accountd.calls() > 0);
}

#[test]
fn a_typed_secret_is_never_sent_to_a_program_that_took_the_name() {
    let rt = runtime();
    let bus = PrivateBus::start("accounts-squatter-secret");
    let squatter = Accountd::new(Policy::Allow, identities());
    let daemon = rt.block_on(squatter.serve_on(&bus));
    let vouch = vouching(&rt, &bus, &daemon, Owner::Nobody);
    let sent = rt.block_on(async {
        let me = bus.connect().await;
        let client =
            LiveClient::new(&me, ACCOUNTS_BUS, ACCOUNTS_SETTINGS_PATH, vouch).expect("a client");
        client
            .set(
                &KeyPath("accounts.clients.microsoft".to_owned()),
                &toml::Value::String("the-client-secret".to_owned()),
            )
            .await
    });
    assert!(matches!(sent, Err(LiveError::NotTheService(_))), "{sent:?}");
    assert_eq!(squatter.calls(), 0);
    assert!(squatter.sets().is_empty(), "the secret did not leave");
}
