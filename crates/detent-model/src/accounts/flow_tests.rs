//! The Accounts pane's sign-in outcomes, rows by state, Add Account and the removal question's
//! clause about kept files.

use super::*;
use crate::picker::Described;
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};
use std::collections::BTreeMap;

pub(super) fn spec(key: &str, label: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        ends: None,
        control: None,
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

pub(super) fn fixed(key: &str, variant: &str) -> KeySpec {
    spec(
        key,
        "row",
        KeyKind::Fixed {
            variant: variant.to_owned(),
        },
    )
}

pub(super) fn action(key: &str, name: &str) -> KeySpec {
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

pub(super) fn toggle(key: &str) -> KeySpec {
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

pub(super) fn ready_with(keys: Vec<KeySpec>, values: &[(&str, &str)]) -> AccountsState {
    accounts_step(
        AccountsState::default(),
        AccountsInput::Found(ModuleId::Accountd, Ok(described(keys, values))),
    )
    .0
}

#[test]
fn each_sign_in_outcome_is_told_and_reads_the_rows_again() {
    let keys = vec![
        fixed("accounts.a.label", "Work mail"),
        action("accounts.a.reauth", "Sign in again"),
    ];
    let state = ready_with(keys, &[]);
    let row = KeyPath("accounts.a.reauth".to_owned());
    let ends = [
        ("done", ReauthEnd::Done),
        ("cancelled", ReauthEnd::Cancelled),
        ("refused", ReauthEnd::Refused),
        ("no_launcher", ReauthEnd::NoLauncher),
        ("failed", ReauthEnd::Failed),
    ];
    for (word, end) in ends {
        let input = AccountsInput::heard(
            ModuleId::Accountd,
            row.clone(),
            &toml::Value::String(word.to_owned()),
        );
        let (_, effects) = accounts_step(state.clone(), input);
        assert_eq!(
            effects,
            [
                AccountsEffect::Tell(AccountsNotice::SignedIn {
                    end,
                    account: Some("Work mail".to_owned()),
                }),
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

pub(super) fn state_of(state: AccountsState) -> AccountsReady {
    match state {
        AccountsState::Ready(ready) => *ready,
        other => panic!("not ready: {other:?}"),
    }
}

#[test]
fn add_account_opens_the_sheet_once_and_each_end_is_told() {
    let state = ready_with(vec![fixed("accounts.a.label", "Home")], &[]);
    let (open, effects) = accounts_step(state, AccountsInput::AddAccount);
    assert_eq!(effects, [AccountsEffect::OpenAddSheet]);
    assert!(state_of(open.clone()).sheet_open);
    // A second press while the sheet is open does nothing.
    let (_, effects) = accounts_step(open.clone(), AccountsInput::AddAccount);
    assert!(effects.is_empty());
    let (done, effects) = accounts_step(
        open.clone(),
        AccountsInput::AddEnded(ModuleId::Accountd, AddEnd::Closed),
    );
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Accountd)]);
    assert!(!state_of(done).sheet_open);
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
            AccountsEffect::Tell(AccountsNotice::AddEnded(AddEnd::AlreadyThere(AccountKey(
                "a".to_owned()
            )))),
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
        AccountsEffect::Tell(AccountsNotice::AddEnded(AddEnd::Failed))
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
    for (word, kept) in [("on", true), ("off", false)] {
        let state = ready_with(keys.clone(), &[("accounts.a.sync.files", word)]);
        let (asked, _) = accounts_step(state, AccountsInput::AskRemove(remove.clone()));
        assert_eq!(state_of(asked).confirming.map(|q| q.files_kept), Some(kept));
    }
}
