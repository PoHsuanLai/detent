//! A call goes to the program that was checked, not to whoever holds accountd's name when the call
//! is made: a registry that loses the name to a second one the moment the link has asked who
//! owns it.

use crate::handover::HandsOver;
use crate::private_bus::PrivateBus;
use crate::spaces_fakes::{Fakes, serve, serve_unnamed};
use almanac_core::Removal;
use detent_model::SpaceKey;
use detent_system::bus::{BusSource, Vouch};
use detent_system::spaces::{OnBus, SpacesLink};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use zbus::fdo::DBusProxy;
use zbus::names::BusName;

const ACCOUNTS: &str = "org.quire.Accounts1";

/// The registry that lists Trip, a second registry that lists Imposter and takes accountd's name
/// from the first once `armed` is set and the link has asked who owns it.
struct World {
    original: Fakes,
    successor: Fakes,
    armed: Arc<AtomicBool>,
    link: OnBus,
    _bus: PrivateBus,
}

async fn world(name: &str) -> World {
    let bus = PrivateBus::start(name);
    let original = serve(&bus, &[("space-1", "Trip")], true).await;
    let successor = serve_unnamed(&bus, &[("space-9", "Imposter")]).await;
    let (check, armed) = HandsOver::new(
        ACCOUNTS,
        original.registry.clone(),
        successor.registry.clone(),
    );
    let link = OnBus(BusSource::Address(bus.address.clone(), Vouch::of(check)));
    World {
        original,
        successor,
        armed,
        link,
        _bus: bus,
    }
}

/// The unique name that owns accountd's name now.
async fn owner(via: &zbus::Connection) -> String {
    let bus = DBusProxy::new(via).await.expect("the bus");
    let name = BusName::try_from(ACCOUNTS).expect("a name");
    bus.get_name_owner(name)
        .await
        .expect("an owner")
        .to_string()
}

fn unique(fakes: &Fakes) -> String {
    fakes
        .registry
        .unique_name()
        .expect("a unique name")
        .to_string()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_list_is_read_from_the_program_that_was_checked() {
    let w = world("owner-list").await;
    assert_eq!(
        owner(&w.original.registry).await,
        unique(&w.original),
        "the control: it owns the name"
    );
    w.armed.store(true, Ordering::SeqCst);
    let rows = w.link.list().await.expect("the registry answers");
    let names: Vec<String> = rows.into_iter().map(|row| row.name).collect();
    assert_eq!(
        names,
        ["Trip"],
        "the program that took the name is not listed"
    );
    assert_eq!(
        owner(&w.original.registry).await,
        unique(&w.successor),
        "the name did change hands after the check"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_removal_goes_to_the_program_that_was_checked() {
    let w = world("owner-remove").await;
    let first = w.link.list().await.expect("the registry answers");
    assert_eq!(first.len(), 1);
    w.armed.store(true, Ordering::SeqCst);
    w.link
        .remove(SpaceKey("space-1".to_owned()), Removal::KEEP_ALL)
        .await
        .expect("removed");
    assert_eq!(
        owner(&w.original.registry).await,
        unique(&w.successor),
        "the name did change hands after the check"
    );
    assert_eq!(
        w.original.asked(),
        [
            r#"memory space-1 {"memories":"move_to_apps","history":"keep"}"#.to_owned(),
            "registry remove space-1".to_owned(),
        ],
        "the program checked was asked"
    );
    assert!(
        w.successor.asked().is_empty(),
        "the program that took the name after the check was asked nothing: {:?}",
        w.successor.asked()
    );
}
