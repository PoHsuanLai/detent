//! The Accounts pane's sign-in outcomes, rows by state, Add Account, sign-in keys and the removal
//! question's clause about kept files.

use super::*;
use crate::picker::Described;
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};
use std::collections::BTreeMap;

fn spec(key: &str, label: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        path: KeyPath(key.to_owned()),
        kind,
        default: toml::Value::Boolean(false),
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Accounts,
        section: Section("Account".to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

fn fixed(key: &str, variant: &str) -> KeySpec {
    spec(
        key,
        "row",
        KeyKind::Fixed {
            variant: variant.to_owned(),
        },
    )
}

fn action(key: &str, name: &str) -> KeySpec {
    spec(
        key,
        name,
        KeyKind::Live {
            action: LiveAction {
                label: ActionLabel(name.to_owned()),
                weight: ActionWeight::Plain,
            },
        },
    )
}

fn toggle(key: &str) -> KeySpec {
    spec(
        key,
        "Keep files",
        KeyKind::Toggle {
            variants: ["on".to_owned(), "off".to_owned()],
        },
    )
}

fn described(keys: Vec<KeySpec>, values: &[(&str, &str)]) -> Described {
    Described {
        keys,
        values: values
            .iter()
            .map(|(k, v)| (KeyPath((*k).to_owned()), (*v).to_owned()))
            .collect::<BTreeMap<_, _>>(),
        ..Default::default()
    }
}

fn account_in(state: &str, extra: Vec<KeySpec>) -> Account {
    let mut keys = vec![
        fixed("accounts.a.label", "Claude"),
        fixed("accounts.a.state", state),
    ];
    keys.extend(extra);
    AccountsPane::of(ModuleId::Accountd, &described(keys, &[]))
        .accounts
        .remove(0)
}

fn ready_with(keys: Vec<KeySpec>, values: &[(&str, &str)]) -> AccountsState {
    accounts_step(
        AccountsState::default(),
        AccountsInput::Found(ModuleId::Accountd, Ok(described(keys, values))),
    )
    .0
}

#[test]
fn each_sign_in_outcome_has_a_calm_line_and_reads_the_rows_again() {
    let keys = vec![
        fixed("accounts.a.label", "Work mail"),
        action("accounts.a.reauth", "Sign in again"),
    ];
    let state = ready_with(keys, &[]);
    let row = KeyPath("accounts.a.reauth".to_owned());
    let lines = [
        ("done", "Signed in to Work mail again."),
        ("cancelled", "Signing in to Work mail was cancelled."),
        ("refused", "The sign-in to Work mail was refused."),
        (
            "no_launcher",
            "The app that runs Work mail is not open. Open it and try again.",
        ),
        ("failed", "Could not sign in to Work mail. Try again later."),
    ];
    for (word, line) in lines {
        let input = AccountsInput::heard(
            ModuleId::Accountd,
            row.clone(),
            &toml::Value::String(word.to_owned()),
        );
        let (_, effects) = accounts_step(state.clone(), input);
        assert_eq!(
            effects,
            [
                AccountsEffect::Tell(line.to_owned()),
                AccountsEffect::Reload(ModuleId::Accountd)
            ],
            "{word}"
        );
    }
    // A word nobody knows is just a change.
    let odd = AccountsInput::heard(
        ModuleId::Accountd,
        row,
        &toml::Value::String("later".to_owned()),
    );
    assert_eq!(odd, AccountsInput::Changed(ModuleId::Accountd));
}

#[test]
fn a_state_change_reads_the_rows_again() {
    let heard = AccountsInput::heard(
        ModuleId::Accountd,
        KeyPath("accounts.a.state".to_owned()),
        &toml::Value::String("needs_login".to_owned()),
    );
    assert_eq!(heard, AccountsInput::Changed(ModuleId::Accountd));
}

#[test]
fn a_program_on_this_computer_gets_no_sign_in_row() {
    let account = account_in(
        "ok",
        vec![
            fixed("accounts.a.place", "this_computer"),
            action("accounts.a.reauth", "Sign in again"),
        ],
    );
    assert_eq!(account.reauth, None);
}

#[test]
fn an_assistant_that_needs_login_signs_in_and_is_signed_out_only_when_signed_in() {
    let needs = account_in(
        "needs_login",
        vec![
            action("accounts.a.reauth", "Sign in"),
            action("accounts.a.sign_out", "Sign out"),
        ],
    );
    assert_eq!(
        needs.reauth.map(|(_, name)| name),
        Some("Sign in".to_owned())
    );
    assert_eq!(needs.sign_out, None);
    let signed = account_in("ok", vec![action("accounts.a.sign_out", "Sign out")]);
    assert!(signed.sign_out.is_some());
    assert_eq!(
        reauth_help(Some(porter_core::SignInWay::Agent)),
        "Asks the assistant to sign itself in, which needs its app to be running."
    );
}

#[test]
fn a_sync_row_keeps_the_modules_help_text() {
    let mut row = toggle("accounts.a.sync.photos");
    row.help = Help("It never reads or deletes anything else.".to_owned());
    let account = account_in("ok", vec![row]);
    assert_eq!(
        account.sync[0].help,
        "It never reads or deletes anything else."
    );
}

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
    assert_eq!(
        effects,
        [AccountsEffect::Tell(
            "Set the sign-in key first, then its secret.".to_owned()
        )]
    );
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

fn state_of(state: AccountsState) -> AccountsReady {
    match state {
        AccountsState::Ready(ready) => *ready,
        other => panic!("not ready: {other:?}"),
    }
}

#[test]
fn add_account_opens_the_sheet_once_and_each_end_is_told_calmly() {
    let state = ready_with(vec![fixed("accounts.a.label", "Home")], &[]);
    let (open, effects) = accounts_step(state, AccountsInput::AddAccount);
    assert_eq!(effects, [AccountsEffect::OpenAddSheet]);
    assert_eq!(state_of(open.clone()).adding, Adding::SheetOpen);
    // A second press while the sheet is open does nothing.
    let (_, effects) = accounts_step(open.clone(), AccountsInput::AddAccount);
    assert!(effects.is_empty());
    let (done, effects) = accounts_step(
        open.clone(),
        AccountsInput::AddEnded(ModuleId::Accountd, AddEnd::Closed),
    );
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Accountd)]);
    assert_eq!(state_of(done).adding, Adding::Idle);
    let (_, effects) = accounts_step(
        open.clone(),
        AccountsInput::AddEnded(
            ModuleId::Accountd,
            AddEnd::AlreadyThere(AccountKey("a".to_owned())),
        ),
    );
    assert_eq!(
        effects,
        [
            AccountsEffect::Tell("This account is already added.".to_owned()),
            AccountsEffect::Reload(ModuleId::Accountd),
            AccountsEffect::Show(AccountKey("a".to_owned())),
        ]
    );
    let (_, effects) = accounts_step(
        open,
        AccountsInput::AddEnded(ModuleId::Accountd, AddEnd::Failed),
    );
    assert_eq!(
        effects[0],
        AccountsEffect::Tell("Could not add an account. Try again later.".to_owned())
    );
}

#[test]
fn the_remove_question_says_kept_files_go_too_only_when_a_sync_row_is_on() {
    let keys = vec![
        fixed("accounts.a.label", "Home cloud"),
        toggle("accounts.a.sync.files"),
        action("accounts.a.remove", "Remove account"),
    ];
    let remove = ModuleRow::new(ModuleId::Accountd, KeyPath("accounts.a.remove".to_owned()));
    for (word, files) in [("on", LocalFiles::Kept), ("off", LocalFiles::None)] {
        let state = ready_with(keys.clone(), &[("accounts.a.sync.files", word)]);
        let (asked, _) = accounts_step(state, AccountsInput::AskRemove(remove.clone()));
        assert_eq!(state_of(asked).confirming.map(|q| q.files), Some(files));
    }
}
