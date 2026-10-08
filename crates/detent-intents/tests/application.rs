//! `org.freedesktop.Application` on `org.quire.Settings`, called the way sill calls it.

mod support;

use detent_intents::{APPLICATION_INTERFACE, APPLICATION_PATH};
use detent_model::{AccountSegment, AccountTarget};
use std::collections::HashMap;
use support::{Told, World};
use zbus::zvariant::{ObjectPath, Value};

const ACCOUNT: &str = "/org/quire/Accounts1/account/work_example";

async fn call(
    world: &World,
    method: &str,
    body: impl serde::Serialize + zbus::zvariant::DynamicType,
) -> zbus::Result<()> {
    let launcher = world.launcher().await;
    launcher
        .call_method(
            Some("org.quire.Settings"),
            APPLICATION_PATH,
            Some(APPLICATION_INTERFACE),
            method,
            &body,
        )
        .await
        .map(|_| ())
}

fn no_data() -> HashMap<&'static str, Value<'static>> {
    HashMap::new()
}

fn detail(segment: &str) -> Told {
    Told::Account(AccountTarget::Detail(AccountSegment(segment.to_owned())))
}

#[tokio::test(flavor = "multi_thread")]
async fn open_account_moves_to_that_accounts_detail() {
    let world = World::start("app-open-account", &[]).await;
    let path = ObjectPath::try_from(ACCOUNT).expect("a path");
    call(
        &world,
        "ActivateAction",
        ("open-account", vec![Value::from(path)], no_data()),
    )
    .await
    .expect("answered");
    assert_eq!(world.window.told()[0], detail("work_example"));
    assert!(matches!(world.window.told()[1], Told::Raised(None)));
}

#[tokio::test(flavor = "multi_thread")]
async fn the_account_may_arrive_as_text_the_way_sill_sends_it() {
    let world = World::start("app-open-account-text", &[]).await;
    call(
        &world,
        "ActivateAction",
        ("open-account", vec![Value::from(ACCOUNT)], no_data()),
    )
    .await
    .expect("answered");
    assert_eq!(world.window.told()[0], detail("work_example"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_path_that_is_no_account_gives_the_list() {
    let world = World::start("app-unknown-account", &[]).await;
    for path in ["/elsewhere", "/org/quire/Accounts1/settings"] {
        call(
            &world,
            "ActivateAction",
            ("open-account", vec![Value::from(path)], no_data()),
        )
        .await
        .expect("answered");
    }
    call(
        &world,
        "ActivateAction",
        ("open-account", Vec::<Value<'_>>::new(), no_data()),
    )
    .await
    .expect("answered");
    let lists = world
        .window
        .told()
        .into_iter()
        .filter(|told| *told == Told::Account(AccountTarget::List))
        .count();
    assert_eq!(lists, 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_action_does_nothing() {
    let world = World::start("app-unknown-action", &[]).await;
    call(
        &world,
        "ActivateAction",
        ("frobnicate", Vec::<Value<'_>>::new(), no_data()),
    )
    .await
    .expect("answered");
    assert_eq!(world.window.told(), []);
}

#[tokio::test(flavor = "multi_thread")]
async fn activate_raises_with_the_token() {
    let world = World::start("app-activate", &[]).await;
    let mut data = no_data();
    data.insert("activation-token", Value::from("tok-1"));
    call(&world, "Activate", (data,)).await.expect("answered");
    call(&world, "Activate", (no_data(),))
        .await
        .expect("answered");
    assert_eq!(
        world.window.told(),
        [Told::Raised(Some("tok-1".to_owned())), Told::Raised(None)]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn open_is_not_supported_and_tells_the_window_nothing() {
    let world = World::start("app-open", &[]).await;
    let refused = call(&world, "Open", (vec!["file:///x"], no_data())).await;
    assert!(refused.is_err());
    assert_eq!(world.window.told(), []);
}
