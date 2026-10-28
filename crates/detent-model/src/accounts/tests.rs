//! The Accounts pane's screens, a module that changes or is gone, and the helpers the other
//! account tests share.

use super::*;
use crate::Absence;
use crate::picker::Described;
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};
use std::collections::BTreeMap;

pub(super) fn path(text: &str) -> KeyPath {
    KeyPath(text.to_owned())
}

/// The row at `text` of accountd's module.
pub(super) fn row(text: &str) -> ModuleRow {
    ModuleRow::new(ModuleId::Accountd, path(text))
}

pub(super) fn spec(key: &str, label: &str, kind: KeyKind) -> KeySpec {
    KeySpec {
        ends: None,
        control: None,
        path: path(key),
        kind,
        default: toml::Value::Boolean(false),
        label: Label(label.to_owned()),
        help: Help(String::new()),
        page: Page::Accounts,
        section: Section("Nextcloud".to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
        agent: AgentSetting::HandsOff,
    }
}

pub(super) fn fixed(key: &str, label: &str, variant: &str) -> KeySpec {
    spec(
        key,
        label,
        KeyKind::Fixed {
            variant: variant.to_owned(),
        },
    )
}

pub(super) fn toggle(key: &str, kind: &str) -> KeySpec {
    spec(
        key,
        kind,
        KeyKind::Toggle {
            variants: ["on".to_owned(), "off".to_owned()],
        },
    )
}

/// A grant row: labelled with the sentence, its action named `Revoke`.
pub(super) fn grant() -> KeySpec {
    KeySpec {
        kind: KeyKind::Live {
            action: LiveAction {
                label: ActionLabel("Revoke".to_owned()),
                weight: ActionWeight::Plain,
            },
        },
        ..spec(
            "accounts.home.grant.g1",
            "Photos allowed: storage",
            KeyKind::Text,
        )
    }
}

pub(super) fn action(key: &str, label: &str, weight: ActionWeight) -> KeySpec {
    spec(
        key,
        label,
        KeyKind::Live {
            action: LiveAction {
                label: ActionLabel(label.to_owned()),
                weight,
            },
        },
    )
}

/// Two accounts, one with a dotted id, as accountd would describe them.
pub(super) fn described() -> Described {
    let keys = vec![
        fixed("accounts.home.label", "Account", "Home cloud"),
        fixed("accounts.home.state", "Status", "ok"),
        toggle("accounts.home.service.storage", "storage"),
        toggle("accounts.home.service.calendar", "calendar"),
        toggle(
            "accounts.home.sync.files",
            "Keep this account's files on this computer",
        ),
        toggle(
            "accounts.home.sync.photos",
            "Upload new photos from a folder on this computer to Google Photos",
        ),
        toggle("accounts.home.sync.other", "other"),
        grant(),
        action("accounts.home.reauth", "Sign in again", ActionWeight::Plain),
        action(
            "accounts.home.remove",
            "Remove account",
            ActionWeight::Destructive,
        ),
        fixed("accounts.work.example.label", "Account", "Work mail"),
        fixed("accounts.work.example.state", "Status", "needs_reauth"),
        toggle("accounts.work.example.service.mail", "mail"),
        action(
            "accounts.work.example.remove",
            "Remove account",
            ActionWeight::Destructive,
        ),
        spec(
            "accounts.clients.microsoft",
            "microsoft client id",
            KeyKind::Text,
        ),
    ];
    let values: BTreeMap<KeyPath, String> = [
        ("accounts.home.service.storage", "on"),
        ("accounts.home.service.calendar", "off"),
        ("accounts.home.sync.photos", "on"),
        ("accounts.work.example.service.mail", "on"),
        ("accounts.clients.microsoft", "abc"),
    ]
    .into_iter()
    .map(|(key, word)| (path(key), word.to_owned()))
    .collect();
    Described {
        keys,
        values,
        ..Default::default()
    }
}

pub(super) fn ready() -> AccountsState {
    accounts_step(
        AccountsState::default(),
        AccountsInput::Found(ModuleId::Accountd, Ok(described())),
    )
    .0
}

pub(super) fn state_of(state: AccountsState) -> AccountsReady {
    match state {
        AccountsState::Ready(ready) => *ready,
        other => panic!("not ready: {other:?}"),
    }
}

pub(super) fn apply(
    state: AccountsState,
    inputs: impl IntoIterator<Item = AccountsInput>,
) -> (AccountsState, Vec<AccountsEffect>) {
    inputs
        .into_iter()
        .fold((state, Vec::new()), |(state, mut effects), input| {
            let (next, more) = accounts_step(state, input);
            effects.extend(more);
            (next, effects)
        })
}

#[test]
fn the_pane_opens_on_the_list_and_an_account_chosen_shows_beside_it_until_none_is() {
    for input in [
        AccountsInput::Open(AccountKey("home".to_owned())),
        AccountsInput::Select(AccountKey("home".to_owned())),
    ] {
        let (state, effects) = apply(ready(), [input.clone()]);
        assert!(effects.is_empty(), "{input:?}");
        let up = state_of(state.clone());
        assert_eq!(up.open().map(|a| a.label.as_str()), Some("Home cloud"));
        assert_eq!(
            up.selected,
            Some(AccountKey("home".to_owned())),
            "{input:?}"
        );
        let back = state_of(apply(state, [AccountsInput::Back]).0);
        assert_eq!(back.detail, None);
        assert_eq!(back.selected, None);
    }
}

#[test]
fn an_account_nobody_listed_does_not_open() {
    let (state, _) = apply(
        ready(),
        [AccountsInput::Open(AccountKey("nobody".to_owned()))],
    );
    assert_eq!(state_of(state).detail, None);
}

#[test]
fn a_reading_keeps_the_screen_when_its_account_is_still_there() {
    let (state, _) = apply(
        ready(),
        [
            AccountsInput::Open(AccountKey("work.example".to_owned())),
            AccountsInput::Found(ModuleId::Accountd, Ok(described())),
        ],
    );
    assert_eq!(
        state_of(state).detail,
        Some(AccountKey("work.example".to_owned()))
    );
}

#[test]
fn a_change_the_module_announces_reads_it_again() {
    let (_, effects) = apply(ready(), [AccountsInput::Changed(ModuleId::Accountd)]);
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Accountd)]);
}

#[test]
fn the_pane_is_unavailable_only_when_every_module_is_not_there_and_ignores_the_person() {
    let gone = |module| AccountsInput::Found(module, Err(Absence::NotRunning));
    let (state, effects) = apply(
        AccountsState::default(),
        [
            gone(ModuleId::Accountd),
            gone(ModuleId::Inferd),
            gone(ModuleId::Syncd),
            AccountsInput::Open(AccountKey("home".to_owned())),
        ],
    );
    assert!(matches!(
        state,
        AccountsState::Unavailable(Absence::NotRunning, _)
    ));
    assert!(effects.is_empty());
}
