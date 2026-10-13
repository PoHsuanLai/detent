//! The pane over three modules: each module's rows stay tied to it, one that is gone leaves only
//! its part out, and the same id in two modules is two accounts.

use super::*;
use crate::picker::{Absence, Described};
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};
use std::collections::BTreeMap;

fn spec(key: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        path: KeyPath(key.to_owned()),
        kind,
        default: toml::Value::Boolean(false),
        label: Label("row".to_owned()),
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
        KeyKind::Fixed {
            variant: variant.to_owned(),
        },
    )
}

fn action(key: &str, name: &str) -> KeySpec {
    spec(
        key,
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
        KeyKind::Toggle {
            variants: ["on".to_owned(), "off".to_owned()],
        },
    )
}

fn module(keys: Vec<KeySpec>, values: &[(&str, &str)]) -> Described {
    Described {
        keys,
        values: values
            .iter()
            .map(|(k, v)| (KeyPath((*k).to_owned()), (*v).to_owned()))
            .collect::<BTreeMap<_, _>>(),
    }
}

/// accountd: an identity two apps use.
fn accountd() -> Described {
    module(
        vec![
            fixed("accounts.me.label", "Me"),
            fixed("accounts.me.used_by.0", "Photos"),
            fixed("accounts.me.used_by.1", "Mail"),
            action("accounts.me.remove", "Remove account"),
        ],
        &[],
    )
}

/// inferd: an AI key with one permit.
fn inferd() -> Described {
    module(
        vec![
            fixed("accounts.key.label", "Claude key"),
            fixed("accounts.key.group", "intelligence"),
            action("accounts.key.grant.g1", "Revoke"),
        ],
        &[],
    )
}

/// syncd: the same identity signed in for sync, and a row to continue as another.
fn syncd() -> Described {
    module(
        vec![
            fixed("accounts.me.label", "Me (sync)"),
            toggle("accounts.me.sync.files"),
            action("accounts.me.remove", "Remove account"),
            action("sync.add", "Continue as\u{2026}"),
        ],
        &[("accounts.me.sync.files", "off")],
    )
}

fn heard_all() -> AccountsState {
    let inputs = [
        AccountsInput::Found(ModuleId::Accountd, Ok(accountd())),
        AccountsInput::Found(ModuleId::Inferd, Ok(inferd())),
        AccountsInput::Found(ModuleId::Syncd, Ok(syncd())),
    ];
    inputs
        .into_iter()
        .fold(AccountsState::default(), |state, input| {
            accounts_step(state, input).0
        })
}

fn ready(state: AccountsState) -> AccountsReady {
    match state {
        AccountsState::Ready(ready) => *ready,
        other => panic!("not ready: {other:?}"),
    }
}

#[test]
fn each_account_keeps_the_module_it_came_from_and_the_same_id_is_two_accounts() {
    let pane = ready(heard_all()).pane;
    let found: Vec<(ModuleId, &str)> = pane
        .accounts
        .iter()
        .map(|a| (a.module, a.label.as_str()))
        .collect();
    assert_eq!(
        found,
        [
            (ModuleId::Accountd, "Me"),
            (ModuleId::Inferd, "Claude key"),
            (ModuleId::Syncd, "Me (sync)"),
        ]
    );
    assert_ne!(pane.accounts[0].key, pane.accounts[2].key);
    assert_eq!(pane.accounts[2].key.id(), "me");
}

#[test]
fn an_identity_lists_the_apps_using_it_and_has_no_revoke_row() {
    let pane = ready(heard_all()).pane;
    assert_eq!(pane.accounts[0].used_by, ["Photos", "Mail"]);
    assert!(pane.accounts[0].permissions.is_empty());
}

#[test]
fn the_permits_of_an_ai_key_are_inferds_revoke_rows() {
    let pane = ready(heard_all()).pane;
    let key = &pane.accounts[1];
    assert_eq!(key.permissions.len(), 1);
    assert_eq!(key.permissions[0].action, "Revoke");
    assert_eq!(key.row(&key.permissions[0].path).module, ModuleId::Inferd);
}

#[test]
fn a_switch_and_an_action_go_back_to_the_module_that_has_the_row() {
    let files = ModuleRow::new(
        ModuleId::Syncd,
        KeyPath("accounts.me.sync.files".to_owned()),
    );
    let (state, effects) = accounts_step(
        heard_all(),
        AccountsInput::Switched {
            row: files.clone(),
            on: true,
        },
    );
    assert_eq!(
        effects,
        [AccountsEffect::Set {
            row: files,
            value: toml::Value::String("on".to_owned())
        }]
    );
    assert!(ready(state).pane.accounts[2].sync[0].on);
    // The same path in accountd is another row: the identity has no such switch.
    let elsewhere = ModuleRow::new(
        ModuleId::Accountd,
        KeyPath("accounts.me.sync.files".to_owned()),
    );
    let (_, effects) = accounts_step(
        heard_all(),
        AccountsInput::Switched {
            row: elsewhere,
            on: true,
        },
    );
    assert!(effects.is_empty());
}

#[test]
fn the_same_remove_path_asks_about_the_account_of_its_own_module() {
    let path = KeyPath("accounts.me.remove".to_owned());
    for (module, label) in [(ModuleId::Accountd, "Me"), (ModuleId::Syncd, "Me (sync)")] {
        let (state, _) = accounts_step(
            heard_all(),
            AccountsInput::AskRemove(ModuleRow::new(module, path.clone())),
        );
        let asked = ready(state).confirming.expect("a question");
        assert_eq!(asked.account, label);
        assert_eq!(asked.row.module, module);
    }
}

#[test]
fn continue_as_runs_on_one_press_in_syncd() {
    let add = ready(heard_all()).pane.sync_add.expect("the row");
    let (state, effects) = accounts_step(heard_all(), AccountsInput::Ran(add.row()));
    assert_eq!(effects, [AccountsEffect::Invoke(add.row())]);
    assert_eq!(ready(state).pending, Some(add.row()));
}

#[test]
fn a_pane_with_only_accountd_has_no_ai_or_sync_part() {
    let (state, _) = accounts_step(
        AccountsState::default(),
        AccountsInput::Found(ModuleId::Accountd, Ok(accountd())),
    );
    let pane = ready(state).pane;
    assert_eq!(pane.accounts.len(), 1);
    assert!(pane.has(ModuleId::Accountd));
    assert!(!pane.has(ModuleId::Inferd) && !pane.has(ModuleId::Syncd));
    assert!(pane.sync_add.is_none());
}

#[test]
fn a_module_that_vanishes_leaves_the_others_shown() {
    let (state, _) = accounts_step(
        heard_all(),
        AccountsInput::Found(ModuleId::Syncd, Err(Absence::NotRunning)),
    );
    let up = ready(state);
    assert_eq!(up.pane.accounts.len(), 2);
    assert!(up.pane.sync_add.is_none());
    assert!(!up.pane.has(ModuleId::Syncd));
}

#[test]
fn the_screen_of_a_vanished_modules_account_goes_back_to_the_list() {
    let sync_key = AccountKey::of(ModuleId::Syncd, "me");
    let (state, _) = accounts_step(heard_all(), AccountsInput::Open(sync_key));
    assert!(matches!(
        ready(state.clone()).screen,
        AccountsScreen::Detail(_)
    ));
    let (state, _) = accounts_step(
        state,
        AccountsInput::Found(ModuleId::Syncd, Err(Absence::NotRunning)),
    );
    assert_eq!(ready(state).screen, AccountsScreen::List);
}

#[test]
fn a_failed_reread_of_a_module_retries_that_module_only() {
    let (_, effects) = accounts_step(
        heard_all(),
        AccountsInput::Reread(ModuleId::Inferd, Err(Absence::Failed("moved".to_owned()))),
    );
    assert_eq!(effects, [AccountsEffect::Retry(ModuleId::Inferd)]);
}

#[test]
fn a_change_in_a_module_reads_that_module_again() {
    let (_, effects) = accounts_step(heard_all(), AccountsInput::Changed(ModuleId::Syncd));
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Syncd)]);
}

#[test]
fn adding_an_ai_key_opens_its_own_sheet_and_reads_inferd_again() {
    let (open, effects) = accounts_step(heard_all(), AccountsInput::AddAiKey);
    assert_eq!(effects, [AccountsEffect::OpenAiKeySheet]);
    let (done, effects) = accounts_step(
        open,
        AccountsInput::AddEnded(ModuleId::Inferd, AddEnd::Added),
    );
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Inferd)]);
    assert_eq!(ready(done).adding, Adding::Idle);
}
