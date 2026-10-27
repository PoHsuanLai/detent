//! Settings' account actions, called over a private bus the way intentd calls an app: the list
//! says what an agent may know of the accounts, and nothing in the manifest changes one.

use crate::support::{Listed, World, entity, key};
use detent_model::{
    Absence, Account, AccountKey, AccountService, AccountSpend, AccountStatus, AccountsPane,
    ModuleId, Place, SwitchState,
};
use docket_core::{AppRefusal, Value};
use ds_settings::schema::KeyPath;
use porter_core::sheet::ProviderGroup;
use prov::Integrity;

fn account(id: &str, label: &str, status: AccountStatus, kinds: &[&str]) -> Account {
    Account {
        key: AccountKey(id.to_owned()),
        module: ModuleId::Accountd,
        label: label.to_owned(),
        status,
        place: Place::Elsewhere,
        expires: None,
        sign_in: None,
        provider: None,
        group: ProviderGroup::Internet,
        services: kinds
            .iter()
            .map(|kind| AccountService {
                path: KeyPath(format!("accounts.{id}.service.{kind}")),
                kind: (*kind).to_owned(),
                on: SwitchState::On,
            })
            .collect(),
        sync: Vec::new(),
        show_in_files: None,
        permissions: Vec::new(),
        reauth: Some((
            KeyPath(format!("accounts.{id}.reauth")),
            "Sign in again".to_owned(),
        )),
        sign_out: None,
        remove: Some((
            KeyPath(format!("accounts.{id}.remove")),
            "Remove account".to_owned(),
        )),
        used_by: Vec::new(),
        spend: AccountSpend::default(),
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
        ..AccountsPane::default()
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
    let Value::Text(text) = &told.value else {
        panic!("text");
    };
    assert_eq!(
        text,
        "Home cloud (storage, calendar): Signed in\nWork mail (mail): Needs sign-in"
    );
    assert!(
        !text.contains("secret-id") && !text.contains("accounts."),
        "no id and no key: {text}"
    );
    assert_eq!(told.label.integrity, Integrity::Untrusted);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_long_accounts_list_is_cut_to_the_declared_size_on_a_character() {
    // A label is whatever the service calls the account; these are wide characters, so a cut
    // by bytes would land inside one.
    let wide = "\u{3042}".repeat(60);
    let many = Listed(Ok(AccountsPane {
        accounts: (0..100)
            .map(|n| account(&format!("id-{n}"), &wide, AccountStatus::Ok, &["mail"]))
            .collect(),
        ..AccountsPane::default()
    }));
    let world = World::start_listing("list-accounts-long", &[], many).await;
    let out = world
        .perform("settings.accounts.list", &[])
        .await
        .expect("listed");
    let Value::Text(text) = out.value.expect("a value").value else {
        panic!("text");
    };
    assert_eq!(text.chars().count(), 2000, "the manifest's declared size");
    assert!(text.ends_with('\u{2026}'), "a cut says so");
    assert!(text.starts_with(&wide), "what is kept is the start");
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
async fn an_account_row_cannot_be_read_or_set_as_a_key() {
    let world = World::start_listing("accounts-not-keys", &[], two_accounts()).await;
    for path in [
        "accounts.secret-id-1.service.storage",
        "accounts.secret-id-1.remove",
    ] {
        for action in ["settings.key.read", "settings.key.set"] {
            let args = [("key", key(path)), ("value", crate::support::text("off"))];
            let refusal = world.perform(action, &args).await.expect_err("refused");
            assert_eq!(
                refusal,
                AppRefusal::NotFound(entity("settings.key", path)),
                "{action} {path}"
            );
        }
    }
}
