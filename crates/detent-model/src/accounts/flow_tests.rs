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
    }
}

fn account_in(state: &str, extra: Vec<KeySpec>) -> Account {
    let mut keys = vec![
        fixed("accounts.a.label", "Claude"),
        fixed("accounts.a.state", state),
    ];
    keys.extend(extra);
    AccountsPane::of(&described(keys, &[])).accounts.remove(0)
}

fn ready_with(keys: Vec<KeySpec>, values: &[(&str, &str)]) -> AccountsState {
    accounts_step(
        AccountsState::Probing,
        AccountsInput::Found(Ok(described(keys, values))),
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
        let input = AccountsInput::heard(row.clone(), &toml::Value::String(word.to_owned()));
        let (_, effects) = accounts_step(state.clone(), input);
        assert_eq!(
            effects,
            [
                AccountsEffect::Tell(line.to_owned()),
                AccountsEffect::Reload
            ],
            "{word}"
        );
    }
    // A word nobody knows is just a change.
    let odd = AccountsInput::heard(row, &toml::Value::String("later".to_owned()));
    assert_eq!(odd, AccountsInput::Changed);
}

#[test]
fn a_state_change_reads_the_rows_again() {
    let heard = AccountsInput::heard(
        KeyPath("accounts.a.state".to_owned()),
        &toml::Value::String("needs_login".to_owned()),
    );
    assert_eq!(heard, AccountsInput::Changed);
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

#[test]
fn a_secret_waits_for_its_key_and_says_so() {
    let state = ready_with(google_keys(), &[]);
    let secret = KeyPath("accounts.clients.google.secret".to_owned());
    let pane = state_of(state.clone()).pane;
    assert_eq!(pane.clients[0].entry, ClientEntry::Open);
    assert_eq!(pane.clients[1].entry, ClientEntry::WaitsForKey);
    let (after, effects) = accounts_step(
        state,
        AccountsInput::ClientEdited {
            path: secret.clone(),
            text: "s3cret".to_owned(),
        },
    );
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
    let (_, effects) = accounts_step(
        keyed,
        AccountsInput::ClientEdited {
            path: secret.clone(),
            text: "s3cret".to_owned(),
        },
    );
    assert_eq!(
        effects,
        [AccountsEffect::Set {
            path: secret,
            value: toml::Value::String("s3cret".to_owned())
        }]
    );
}

fn state_of(state: AccountsState) -> AccountsReady {
    match state {
        AccountsState::Ready(ready) => ready,
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
    let (done, effects) = accounts_step(open.clone(), AccountsInput::AddEnded(AddEnd::Closed));
    assert_eq!(effects, [AccountsEffect::Reload]);
    assert_eq!(state_of(done).adding, Adding::Idle);
    let (_, effects) = accounts_step(
        open.clone(),
        AccountsInput::AddEnded(AddEnd::AlreadyThere(AccountKey("a".to_owned()))),
    );
    assert_eq!(
        effects,
        [
            AccountsEffect::Tell("This account is already added.".to_owned()),
            AccountsEffect::Reload,
            AccountsEffect::Show(AccountKey("a".to_owned())),
        ]
    );
    let (_, effects) = accounts_step(open, AccountsInput::AddEnded(AddEnd::Failed));
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
    let remove = KeyPath("accounts.a.remove".to_owned());
    for (word, files) in [("on", LocalFiles::Kept), ("off", LocalFiles::None)] {
        let state = ready_with(keys.clone(), &[("accounts.a.sync.files", word)]);
        let (asked, _) = accounts_step(state, AccountsInput::AskRemove(remove.clone()));
        assert_eq!(state_of(asked).confirming.map(|q| q.files), Some(files));
    }
}
