//! `org.freedesktop.Application` on `org.quire.Settings`, called the way sill calls it.

use crate::support::{World, peers_knowing};
use detent_callers::roles;
use detent_intents::{APPLICATION_INTERFACE, APPLICATION_PATH, HandOverFault, hand_over};
use detent_model::{
    ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, AccountSegment, AccountTarget, Notice, PageSlug,
};
use std::collections::HashMap;
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

fn detail(segment: &str) -> Notice {
    Notice::OpenAccount(AccountTarget::Detail(AccountSegment(segment.to_owned())))
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
    assert!(matches!(world.window.told()[1], Notice::Raise(None)));
}

#[tokio::test(flavor = "multi_thread")]
async fn open_target_tells_the_window_the_target_its_text_names() {
    let world = World::start("app-open-target", &[]).await;
    for text in ["page:dock", "key:dock.magnification", "account:work-example"] {
        call(
            &world,
            "ActivateAction",
            ("open-target", vec![Value::from(text)], no_data()),
        )
        .await
        .expect("answered");
    }
    call(
        &world,
        "ActivateAction",
        ("open-target", vec![Value::from("nowhere")], no_data()),
    )
    .await
    .expect("answered");
    let targets: Vec<String> = world
        .window
        .told()
        .into_iter()
        .filter_map(|notice| match notice {
            Notice::OpenTarget(target) => Some(target.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(
        targets,
        ["page:dock", "key:dock.magnification", "account:work-example"],
        "text that is no target is ignored"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn open_page_shows_the_page_its_slug_names_and_raises() {
    let world = World::start("app-open-page", &[]).await;
    call(
        &world,
        "ActivateAction",
        ("open-page", vec![Value::from("software-update")], no_data()),
    )
    .await
    .expect("answered");
    assert_eq!(
        world.window.told(),
        [
            Notice::OpenPage(PageSlug("software-update".to_owned())),
            Notice::Raise(None)
        ]
    );
    call(
        &world,
        "ActivateAction",
        ("open-page", Vec::<Value<'_>>::new(), no_data()),
    )
    .await
    .expect("answered");
    assert_eq!(
        world.window.told()[2..],
        [Notice::Raise(None)],
        "a page that is not named only raises the window"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn open_page_shows_the_assistants_model_page_and_its_first_run_step() {
    let world = World::start("app-open-assistant-model", &[]).await;
    for slug in [ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG] {
        call(
            &world,
            "ActivateAction",
            ("open-page", vec![Value::from(slug)], no_data()),
        )
        .await
        .expect("answered");
    }
    assert_eq!(
        world.window.told(),
        [
            Notice::OpenPage(PageSlug("assistant-model".to_owned())),
            Notice::Raise(None),
            Notice::OpenPage(PageSlug("assistant-model-setup".to_owned())),
            Notice::Raise(None),
        ],
        "the slugs sill is told to send, each shown and the window raised"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_detent_hands_over_the_page_it_was_asked_for() {
    let world = World::start("app-hand-over", &[]).await;
    let second = world.caller(roles::SETTINGS).await;
    let knows = peers_knowing(&second, &world.owner(), &roles::SETTINGS);
    hand_over(
        &second,
        &knows,
        None,
        Some(PageSlug("software-update".to_owned())),
    )
    .await
    .expect("answered");
    hand_over(&second, &knows, Some("tok-2".to_owned()), None)
        .await
        .expect("answered");
    assert_eq!(
        world.window.told(),
        [
            Notice::OpenPage(PageSlug("software-update".to_owned())),
            Notice::Raise(None),
            Notice::Raise(Some("tok-2".to_owned())),
        ]
    );
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
        .filter(|told| *told == Notice::OpenAccount(AccountTarget::List))
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
        [Notice::Raise(Some("tok-1".to_owned())), Notice::Raise(None)]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn open_is_not_supported_and_tells_the_window_nothing() {
    let world = World::start("app-open", &[]).await;
    let refused = call(&world, "Open", (vec!["file:///x"], no_data())).await;
    assert!(refused.is_err());
    assert_eq!(world.window.told(), []);
}

/// Whether the activation object admits a role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Door {
    Open,
    Shut,
}

/// Every role of the desktop's table, with what the activation object does for it.
fn activation_table() -> Vec<(detent_callers::RoleName, Door)> {
    let admitted = [roles::SHELL, roles::SETTINGS, roles::AGENT_TIER];
    roles::ALL
        .into_iter()
        .map(|role| {
            let door = if admitted.contains(&role) {
                Door::Open
            } else {
                Door::Shut
            };
            (role, door)
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_shell_a_second_settings_and_the_agent_tier_may_activate() {
    let world = World::start("app-activation-roles", &[]).await;
    for (role, door) in activation_table() {
        let before = world.window.told().len();
        let caller = world.caller(role.clone()).await;
        let page = Value::from("software-update");
        let answered = caller
            .call_method(
                Some("org.quire.Settings"),
                APPLICATION_PATH,
                Some(APPLICATION_INTERFACE),
                "ActivateAction",
                &("open-page", vec![page], no_data()),
            )
            .await;
        let told = world.window.told().len() - before;
        match door {
            Door::Open => {
                assert!(answered.is_ok(), "{role} is admitted");
                assert_eq!(told, 2, "{role} opens the page and raises the window");
            }
            Door::Shut => {
                assert!(answered.is_err(), "{role} is refused");
                assert_eq!(told, 0, "{role} tells the window nothing");
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_program_in_no_row_of_the_table_cannot_raise_or_open_anything() {
    let world = World::start("app-activation-stranger", &[]).await;
    let stranger = world.stranger().await;
    let mut data = no_data();
    data.insert("activation-token", Value::from("stolen-focus"));
    let raised = stranger
        .call_method(
            Some("org.quire.Settings"),
            APPLICATION_PATH,
            Some(APPLICATION_INTERFACE),
            "Activate",
            &(data,),
        )
        .await;
    assert!(raised.is_err());
    let path = ObjectPath::try_from(ACCOUNT).expect("a path");
    let opened = stranger
        .call_method(
            Some("org.quire.Settings"),
            APPLICATION_PATH,
            Some(APPLICATION_INTERFACE),
            "ActivateAction",
            &("open-account", vec![Value::from(path)], no_data()),
        )
        .await;
    assert!(opened.is_err());
    assert_eq!(world.window.told(), [], "the window heard nothing");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_detent_sends_nothing_to_an_owner_that_is_not_settings() {
    let world = World::start("app-hand-over-squatter", &[]).await;
    let second = world.caller(roles::SETTINGS).await;
    // The second detent's peers know the owner of the name as the shell, not as Settings.
    let knows = peers_knowing(&second, &world.owner(), &roles::SHELL);
    let handed = hand_over(&second, &knows, Some("tok-3".to_owned()), None).await;
    assert!(
        matches!(handed, Err(HandOverFault::NotSettings(_))),
        "{handed:?}"
    );
    assert_eq!(world.window.told(), [], "no token and no page was sent");
    // And an owner nobody knows is no better.
    let nobody = detent_callers::private_peers(&second);
    let handed = hand_over(&second, &nobody, Some("tok-4".to_owned()), None).await;
    assert!(handed.is_err());
    assert_eq!(world.window.told(), []);
}
