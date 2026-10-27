//! The sign-in keys and secrets: a secret waits for its key, is never kept, and is written only
//! when something was typed.

use super::flow_tests::{ready_with, spec, state_of};
use super::*;
use ds_settings::schema::{KeyKind, KeyPath, KeySpec};

fn google_keys() -> Vec<KeySpec> {
    vec![
        spec(
            "accounts.clients.google",
            "Google sign-in key",
            KeyKind::Text,
        ),
        spec(
            "accounts.clients.google.secret",
            "Google sign-in secret",
            KeyKind::Text,
        ),
    ]
}

const SECRET: &str = "accounts.clients.google.secret";

fn secret_row() -> ModuleRow {
    ModuleRow::new(ModuleId::Accountd, KeyPath(SECRET.to_owned()))
}

fn typed(text: &str) -> AccountsInput {
    AccountsInput::ClientEdited {
        row: secret_row(),
        text: ClientText::Secret(Secret::new(text)),
    }
}

#[test]
fn a_secret_waits_for_its_key_and_says_so() {
    let state = ready_with(google_keys(), &[]);
    let pane = state_of(state.clone()).pane;
    assert_eq!(pane.clients[0].entry, ClientEntry::Open);
    assert_eq!(pane.clients[1].entry, ClientEntry::WaitsForKey);
    let (after, effects) = accounts_step(state, typed("s3cret"));
    assert_eq!(effects, [AccountsEffect::Tell(AccountsNotice::KeyFirst)]);
    assert_eq!(state_of(after).pending, None);
    // With the key set, the secret is a row like the key.
    let keyed = ready_with(google_keys(), &[("accounts.clients.google", "abc")]);
    assert_eq!(
        state_of(keyed.clone()).pane.clients[1].entry,
        ClientEntry::Open
    );
    let (_, effects) = accounts_step(keyed, typed("s3cret"));
    assert_eq!(
        effects,
        [AccountsEffect::SetSecret {
            row: secret_row(),
            secret: Secret::new("s3cret"),
        }]
    );
}

#[test]
fn a_stored_secret_is_never_kept_only_whether_there_is_one() {
    let keys_set = [
        ("accounts.clients.google", "abc"),
        (SECRET, "hunter2-hunter2"),
    ];
    let ready = state_of(ready_with(google_keys(), &keys_set));
    let pane = ready.pane;
    assert_eq!(
        pane.clients[1].value,
        ClientValue::Secret(SecretStanding::Set)
    );
    assert!(!format!("{pane:?}").contains("hunter2"), "{pane:?}");
    assert!(
        !format!("{:?}", ready.sources).contains("hunter2"),
        "the rows the pane keeps hold no secret text"
    );
    let none = state_of(ready_with(
        google_keys(),
        &[("accounts.clients.google", "abc")],
    ))
    .pane;
    assert_eq!(
        none.clients[1].value,
        ClientValue::Secret(SecretStanding::NotSet)
    );
    // The key beside it is drawn as it is.
    assert_eq!(pane.clients[0].value, ClientValue::Text("abc".to_owned()));
}

#[test]
fn typing_replaces_a_secret_and_an_empty_field_changes_nothing() {
    let set = [
        ("accounts.clients.google", "abc"),
        (SECRET, "old-secret-text"),
    ];
    let (replaced, effects) = accounts_step(ready_with(google_keys(), &set), typed("  new  "));
    assert_eq!(
        effects,
        [AccountsEffect::SetSecret {
            row: secret_row(),
            secret: Secret::new("new"),
        }],
        "the same standing, a new text: it is written, trimmed"
    );
    assert_eq!(state_of(replaced).pending, Some(secret_row()));
    let (_, empty) = accounts_step(ready_with(google_keys(), &set), typed("   "));
    assert!(
        empty.is_empty(),
        "leaving the field alone writes nothing: {empty:?}"
    );
}

#[test]
fn clear_removes_a_set_secret_by_writing_nothing_and_only_that() {
    let set = [
        ("accounts.clients.google", "abc"),
        (SECRET, "old-secret-text"),
    ];
    let (cleared, effects) = accounts_step(
        ready_with(google_keys(), &set),
        AccountsInput::SecretCleared(secret_row()),
    );
    assert_eq!(
        effects,
        [AccountsEffect::Set {
            row: secret_row(),
            value: toml::Value::String(String::new()),
        }]
    );
    let cleared = state_of(cleared);
    assert_eq!(cleared.pending, Some(secret_row()));
    assert_eq!(
        cleared.pane.clients[1].value,
        ClientValue::Secret(SecretStanding::NotSet)
    );
    // `(case, the rows, the row pressed)`: nothing is written for a secret there is none of,
    // for a key, or while another change is in flight.
    let key_row = ModuleRow::new(
        ModuleId::Accountd,
        KeyPath("accounts.clients.google".to_owned()),
    );
    let none = [("accounts.clients.google", "abc")];
    let cases = [
        ("no secret", ready_with(google_keys(), &none), secret_row()),
        (
            "a key, not a secret",
            ready_with(google_keys(), &set),
            key_row,
        ),
        (
            "a change in flight",
            accounts_step(ready_with(google_keys(), &set), typed("new")).0,
            secret_row(),
        ),
    ];
    for (case, state, row) in cases {
        let (_, effects) = accounts_step(state, AccountsInput::SecretCleared(row));
        assert!(effects.is_empty(), "{case}: {effects:?}");
    }
}

#[test]
fn a_secret_never_prints_in_a_debug_line() {
    const TEXT: &str = "hunter2-hunter2";
    let secret = Secret::new(TEXT);
    let input = typed(TEXT);
    let (_, effects) = accounts_step(
        ready_with(google_keys(), &[("accounts.clients.google", "abc")]),
        input.clone(),
    );
    for line in [
        format!("{secret:?}"),
        format!("{:?}", ClientText::Secret(Secret::new(TEXT))),
        format!("{input:?}"),
        format!("{effects:?}"),
        format!("{:#?}", effects),
    ] {
        assert!(!line.contains("hunter2"), "{line}");
        assert!(line.contains("<hidden>"), "{line}");
    }
    assert_eq!(effects.len(), 1, "the effect that was printed is the write");
}
