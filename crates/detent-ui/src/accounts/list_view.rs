//! The list of accounts, and the sign-in client ids under it.

use super::adapter::{
    ActionButton, AddAccountButton, AddKeyButton, ClientField, RemoveAccountButton,
};
use super::avatar_view::{AccountAvatar, AvatarPlace};
use crate::copy::accounts::{attention_badge, status_label};
use crate::key_row::row_id;
use crate::marks::ProviderMarks;
use detent_model::external::ProviderGroup;
use detent_model::{
    Account, AccountKey, AccountStatus, AccountsPane as Pane, Attention, ClientText, ClientValue,
    ModuleId, ModuleRow, SyncBook,
};
use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::badge::{Badge, BadgeContent, BadgeTone};
use ds::components::controls::disclosure::Disclosure;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::lists::list::model::ListStyle;
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::ActionWeight;

/// What the list's foot does: add an account, or remove the one the list rests on.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct FootHandlers {
    /// The plus was pressed.
    pub add: EventHandler<()>,
    /// Add AI Key was pressed.
    pub add_key: EventHandler<()>,
    /// The sync service's row that continues as a new account was pressed.
    pub continue_as: EventHandler<ModuleRow>,
    /// The minus was pressed.
    pub remove: EventHandler<()>,
}

/// The foot under the list: a plus that adds an account and a minus that removes the selected
/// one, which waits while nothing is selected or the selection cannot be removed.
#[component]
fn ListFoot(
    sheet_open: bool,
    availability: Availability,
    removal: Availability,
    on: FootHandlers,
) -> Element {
    rsx! {
        div { class: "accounts-add", "data-accounts": "add",
            AddAccountButton { sheet_open, availability, onpress: move |()| on.add.call(()) }
            RemoveAccountButton { availability: removal, onpress: move |()| on.remove.call(()) }
        }
    }
}

/// The account group's rows as list items: a heading, then a row each; the sync service's
/// accounts come last under a heading of their own.
fn items_of(
    pane: &Pane,
    selected: Option<&AccountKey>,
    marks: &ProviderMarks,
    book: &SyncBook,
    onselect: EventHandler<AccountKey>,
) -> Vec<ListItem<String>> {
    let grouped = ProviderGroup::ALL.into_iter().flat_map(|group| {
        let accounts: Vec<&Account> = pane
            .in_group(group)
            .filter(|account| account.module != ModuleId::Syncd)
            .collect();
        let heading = (!accounts.is_empty()).then(|| {
            ListItem::heading(
                format!("group:{}", group.slug()),
                rsx! { SectionHeader { title: crate::copy::accounts::group_heading(group).to_owned() } },
            )
        });
        let rows = accounts
            .into_iter()
            .map(|account| account_item(account, selected, marks, book, onselect));
        heading.into_iter().chain(rows).collect::<Vec<_>>()
    });
    let synced: Vec<&Account> = pane
        .accounts
        .iter()
        .filter(|account| account.module == ModuleId::Syncd)
        .collect();
    let heading = (!synced.is_empty()).then(|| {
        ListItem::heading(
            "group:sync".to_owned(),
            rsx! { SectionHeader { title: crate::copy::account_views::SYNC.to_owned() } },
        )
    });
    let sync_rows = synced
        .into_iter()
        .map(|account| account_item(account, selected, marks, book, onselect));
    grouped.chain(heading).chain(sync_rows).collect::<Vec<_>>()
}

/// The line under an account's name: the kind of account it is ("Fastmail"), or how it stands
/// when the module names no kind.
fn second_line(account: &Account) -> String {
    account.provider.as_ref().map_or_else(
        || status_label(&account.status),
        |provider| provider.name.clone(),
    )
}

/// How an account that is not fine stands, or what is coming to one that is; nothing for one that
/// is fine.
fn standing(status: &AccountStatus, attention: Option<Attention>) -> Option<String> {
    match (status, attention) {
        (AccountStatus::Ok, None) => None,
        (AccountStatus::Ok, Some(attention)) => Some(attention_badge(&attention).to_owned()),
        _ => Some(status_label(status)),
    }
}

/// The row's name as a screen reader says it: the account, its second line, and how it stands
/// when its dot shows ("ada@example.com, Microsoft, Needs sign-in").
fn row_name(account: &Account) -> String {
    let mut words = vec![account.label.clone(), second_line(account)];
    words.extend(standing(&account.status, Attention::of(account)));
    words.join(", ")
}

/// A dot after the words of an account that is not fine, named by how it stands; an account that
/// is fine but about to be signed out has one too, named by what is coming.
#[component]
fn StandingDot(status: AccountStatus, attention: Option<Attention>) -> Element {
    let Some(name) = standing(&status, attention) else {
        return rsx! {};
    };
    rsx! {
        Badge {
            content: BadgeContent::Dot,
            tone: BadgeTone::Alert,
            size: ControlSize::Mini,
            common: Common { aria_label: Some(name), ..Common::default() },
        }
    }
}

fn account_item(
    account: &Account,
    selected: Option<&AccountKey>,
    marks: &ProviderMarks,
    book: &SyncBook,
    onselect: EventHandler<AccountKey>,
) -> ListItem<String> {
    let key = account.key.clone();
    let state = RowState {
        selection: if selected == Some(&key) {
            Selection::Selected
        } else {
            Selection::Unselected
        },
        ..RowState::default()
    };
    let content = rsx! {
        div { class: "account-row-body",
            AccountAvatar {
                label: account.label.clone(),
                provider: account.provider.clone(),
                marks: marks.clone(),
                place: AvatarPlace::Row,
                syncing: book.syncing(account),
            }
            div { class: "account-row-words",
                Label { text: account.label.clone(), role: LabelRole::Primary }
                Label { text: second_line(account), role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
            StandingDot { status: account.status.clone(), attention: Attention::of(account) }
        }
    };
    let picked = key.clone();
    ListItem::row(
        key.0.clone(),
        account.label.clone(),
        rsx! {
            Row {
                title: account.label.clone(),
                content,
                state,
                common: Common {
                    id: Some(row_id("account", &key.0)),
                    aria_label: Some(row_name(account)),
                    ..Common::default()
                },
                onclick: move |_| onselect.call(picked.clone()),
            }
        },
    )
}

/// The accounts in groups, one row each, and the foot that adds and removes; the empty state
/// when there are none. A press on a row chooses it, and its detail shows beside the list.
#[component]
pub(super) fn AccountList(
    pane: Pane,
    selected: Option<AccountKey>,
    sheet_open: bool,
    availability: Availability,
    marks: ProviderMarks,
    book: SyncBook,
    onselect: EventHandler<AccountKey>,
    on: FootHandlers,
) -> Element {
    let removal = match selected.as_ref().and_then(|key| pane.account(key)) {
        Some(account) if account.remove.is_some() => availability,
        _ => Availability::Disabled,
    };
    if pane.accounts.is_empty() {
        return rsx! {
            div { class: "accounts-empty", "data-accounts": "empty",
                EmptyState {
                    title: crate::copy::account_views::NO_ACCOUNTS,
                    description: crate::copy::account_views::NO_ACCOUNTS_HINT,
                }
            }
            ListFoot { sheet_open, availability, removal, on }
            MoreRows { pane, sheet_open, availability, on }
        };
    }
    let items = items_of(&pane, selected.as_ref(), &marks, &book, onselect);
    rsx! {
        div { class: "accounts-list", "data-accounts": "list",
            List::<String> {
                label: crate::copy::common::ACCOUNTS,
                items,
                style: ListStyle::Grouped,
                cursor: selected.as_ref().map(|key| key.0.clone()),
                onselect: move |id: String| onselect.call(AccountKey(id)),
                onpick: move |id: String| onselect.call(AccountKey(id)),
            }
            ListFoot { sheet_open, availability, removal, on }
            MoreRows { pane: pane.clone(), sheet_open, availability, on }
        }
    }
}

/// The rows under the list that add what the foot's plus does not: an AI key (when the key
/// service is there) and, for sync, a new account to continue as (when the sync service offers
/// one).
#[component]
fn MoreRows(pane: Pane, sheet_open: bool, availability: Availability, on: FootHandlers) -> Element {
    let key_row = pane.has(ModuleId::Inferd);
    let continue_row = pane.sync_add.clone();
    if !key_row && continue_row.is_none() {
        return rsx! {};
    }
    rsx! {
        div { class: "accounts-more", "data-accounts": "more",
            FieldGroup {
                if key_row {
                    FieldRow {
                        label: TextLine::from(crate::copy::account_views::AI_KEYS),
                        help: Some(TextLine::from(crate::copy::account_views::ADD_A_KEY_OR_SIGN_IN)),
                        AddKeyButton { sheet_open, availability, onpress: move |()| on.add_key.call(()) }
                    }
                }
                if let Some(add) = continue_row {
                    FieldRow {
                        label: TextLine::from(crate::copy::account_views::SYNC),
                        help: Some(TextLine::from(crate::copy::account_views::CHOOSE_AN_ACCOUNT_TO_SYNC)),
                        ActionButton {
                            label: add.name.clone(),
                            subject: String::new(),
                            weight: ActionWeight::Plain,
                            availability,
                            onpress: {
                                let row = add.row();
                                move |()| on.continue_as.call(row.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

/// The bring-your-own client ids, under a disclosure of their own (the page's Advanced one holds
/// the file's keys); nothing when the account service lists none.
#[component]
pub(super) fn ClientIds(
    pane: Pane,
    onedit: EventHandler<(ModuleRow, ClientText)>,
    onclear: EventHandler<ModuleRow>,
) -> Element {
    let mut opened = use_signal(|| Shown::Hidden);
    if pane.clients.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "accounts-clients", "data-accounts": "clients",
            Disclosure {
                common: Common { id: Some("sign-in-clients".to_owned()), ..Common::default() },
                label: Some(crate::copy::account_views::SIGN_IN_KEYS.to_owned()),
                shown: opened(),
                on_toggle: move |next| opened.set(next),
                FieldGroup {
                    for client in pane.clients.iter().cloned() {
                        FieldRow {
                            key: "{client.path.0}",
                            label: TextLine::from(client.label.clone()),
                            help: (!client.help.is_empty()).then(|| TextLine::from(client.help.clone())),
                            common: Common { id: Some(row_id("client", &client.path.0)), ..Common::default() },
                            ClientField {
                                key: "{client.path.0}/{client.value.key()}",
                                label: client.label.clone(),
                                value: client.value.clone(),
                                availability: if client.waits_for_key {
                                    Availability::Disabled
                                } else {
                                    Availability::Enabled
                                },
                                onchange: {
                                    let row = ModuleRow::new(client.module, client.path.clone());
                                    let held = client.value.clone();
                                    move |text| onedit.call((row.clone(), held.typed(text)))
                                },
                            }
                            if client.value == (ClientValue::Secret { set: true }) {
                                ActionButton {
                                    label: crate::copy::account_views::CLEAR.to_owned(),
                                    subject: client.label.clone(),
                                    weight: ActionWeight::Plain,
                                    availability: Availability::Enabled,
                                    onpress: {
                                        let row = ModuleRow::new(client.module, client.path.clone());
                                        move |()| onclear.call(row.clone())
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A line under a group that says what the group is for.
#[component]
pub(super) fn Note(text: String) -> Element {
    rsx! {
        div { class: "accounts-note",
            Label { text, role: LabelRole::Tertiary, style: LabelStyle::Footnote }
        }
    }
}
