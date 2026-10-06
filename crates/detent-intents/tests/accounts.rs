//! Settings' account actions, called over a private bus the way intentd calls an app: the list
//! says what an agent may know of the accounts, and nothing in the manifest changes one.

mod support;

use detent_intents::manifest;
use detent_model::{Absence, Account, AccountKey, AccountService, AccountStatus, AccountsPane};
use docket_core::{AppRefusal, Value};
use ds_settings::schema::KeyPath;
use prov::Effect;
use prov::Integrity;
use support::{Listed, Told, World, entity, key};

fn account(id: &str, label: &str, status: AccountStatus, kinds: &[&str]) -> Account {
    Account {
        key: AccountKey(id.to_owned()),
        label: label.to_owned(),
        status,
        services: kinds
            .iter()
            .map(|kind| AccountService {
                path: KeyPath(format!("accounts.{id}.service.{kind}")),
                kind: (*kind).to_owned(),
                on: true,
            })
            .collect(),
        permissions: Vec::new(),
        reauth: Some((
            KeyPath(format!("accounts.{id}.reauth")),
            "Sign in again".to_owned(),
        )),
        remove: Some((
            KeyPath(format!("accounts.{id}.remove")),
            "Remove account".to_owned(),
        )),
    }
}

fn two_accounts() -> Listed {
    Listed(Ok(AccountsPane {
        accounts: vec![
            account(
                "secret-id-1",
                "Home cloud",
                AccountStatus::Ok,
                &["storage", "calendar"],
            ),
            account(
                "secret-id-2",
                "Work mail",
                AccountStatus::NeedsReauth,
                &["mail"],
            ),
        ],
        clients: Vec::new(),
    }))
}

#[tokio::test(flavor = "multi_thread")]
async fn listing_the_accounts_gives_labels_kinds_and_state_and_trusts_none_of_it() {
    let world = World::start_listing("list-accounts", &[], two_accounts()).await;
    let out = world
        .perform("settings.accounts.list", &[])
        .await
        .expect("listed");
    let told = out.value.expect("a value");
    assert_eq!(
        told.value,
        Value::Text(
            "Home cloud (storage, calendar): Signed in\nWork mail (mail): Needs sign-in".to_owned()
        )
    );
    assert_eq!(told.label.integrity, Integrity::Untrusted);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_list_carries_no_id_and_no_key() {
    let world = World::start_listing("list-no-ids", &[], two_accounts()).await;
    let out = world
        .perform("settings.accounts.list", &[])
        .await
        .expect("listed");
    let Some(Value::Text(text)) = out.value.map(|v| v.value) else {
        panic!("text");
    };
    assert!(!text.contains("secret-id"), "{text}");
    assert!(!text.contains("accounts."), "{text}");
}

#[tokio::test(flavor = "multi_thread")]
async fn no_accountd_is_a_refusal_with_words() {
    let world =
        World::start_listing("list-no-accountd", &[], Listed(Err(Absence::NotRunning))).await;
    let refusal = world
        .perform("settings.accounts.list", &[])
        .await
        .expect_err("refused");
    let AppRefusal::Failed(why) = refusal else {
        panic!("{refusal:?}");
    };
    assert!(why.0.contains("not running"), "{}", why.0);
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_the_accounts_tells_the_window_to_show_the_page() {
    let world = World::start("open-accounts", &[]).await;
    world
        .perform("settings.accounts.open", &[])
        .await
        .expect("opened");
    assert_eq!(world.window.told(), [Told::Page("accounts".to_owned())]);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_manifest_offers_no_way_to_change_an_account() {
    let manifest = manifest().expect("a manifest");
    let accounts: Vec<_> = manifest
        .manifest()
        .actions
        .iter()
        .filter(|action| action.name.as_str().starts_with("settings.accounts."))
        .collect();
    let names: Vec<&str> = accounts.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["settings.accounts.list", "settings.accounts.open"]);
    assert!(accounts.iter().all(|action| action.effect == Effect::Read));
}

#[tokio::test(flavor = "multi_thread")]
async fn an_account_row_cannot_be_read_or_set_as_a_key() {
    let world = World::start_listing("accounts-not-keys", &[], two_accounts()).await;
    for path in [
        "accounts.secret-id-1.service.storage",
        "accounts.secret-id-1.remove",
    ] {
        for action in ["settings.key.read", "settings.key.set"] {
            let args = [("key", key(path)), ("value", support::text("off"))];
            let refusal = world.perform(action, &args).await.expect_err("refused");
            assert_eq!(
                refusal,
                AppRefusal::NotFound(entity("settings.key", path)),
                "{action} {path}"
            );
        }
    }
}
