//! The list of accounts, and the sign-in client ids under it.

use super::adapter::{AddAccountButton, ClientField, OpenButton, RemoveAccountButton};
use crate::marks::ProviderMarks;
use detent_model::{Account, AccountKey, AccountsPane as Pane, Adding, ClientEntry};
use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::disclosure::Disclosure;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::lists::list::model::ListStyle;
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::KeyPath;
use porter_core::sheet::ProviderGroup;

/// What the list's foot does: add an account, or remove the one the list rests on.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct FootHandlers {
    /// The plus was pressed.
    pub add: EventHandler<()>,
    /// The minus was pressed.
    pub remove: EventHandler<()>,
}

/// The foot under the list: a plus that adds an account and a minus that removes the selected
/// one, which waits while nothing is selected or the selection cannot be removed.
#[component]
fn ListFoot(
    adding: Adding,
    availability: Availability,
    removal: Availability,
    on: FootHandlers,
) -> Element {
    rsx! {
        div { class: "accounts-add", "data-accounts": "add",
            AddAccountButton { adding, availability, onpress: move |()| on.add.call(()) }
            RemoveAccountButton { availability: removal, onpress: move |()| on.remove.call(()) }
        }
    }
}

/// The account group's rows as list items: a heading, then a row each.
fn items_of(
    pane: &Pane,
    selected: Option<&AccountKey>,
    marks: &ProviderMarks,
    onselect: EventHandler<AccountKey>,
    onopen: EventHandler<AccountKey>,
) -> Vec<ListItem<String>> {
    ProviderGroup::ALL
        .into_iter()
        .flat_map(|group| {
            let accounts: Vec<&Account> = pane.in_group(group).collect();
            let heading = (!accounts.is_empty()).then(|| {
                ListItem::heading(
                    format!("group:{}", group.slug()),
                    rsx! { SectionHeader { title: group.display_name().to_owned() } },
                )
            });
            let rows = accounts
                .into_iter()
                .map(|account| account_item(account, selected, marks, onselect, onopen));
            heading.into_iter().chain(rows).collect::<Vec<_>>()
        })
        .collect()
}

fn account_item(
    account: &Account,
    selected: Option<&AccountKey>,
    marks: &ProviderMarks,
    onselect: EventHandler<AccountKey>,
    onopen: EventHandler<AccountKey>,
) -> ListItem<String> {
    let key = account.key.clone();
    let provider = account.provider.as_ref();
    let mark = provider.map(|p| marks.of(&p.id));
    let kind = provider.map_or_else(String::new, |p| p.name.clone());
    let standing = account.status.label();
    let detail = if kind.is_empty() {
        standing
    } else {
        format!("{kind}, {standing}")
    };
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
            if let Some(mark) = mark {
                ProviderMark { provider: mark.provider, face: mark.face, size: ControlSize::Regular }
            }
            div { class: "account-row-words",
                Label { text: account.label.clone(), role: LabelRole::Primary }
                Label { text: detail, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
        }
    };
    let (picked, opened) = (key.clone(), key.clone());
    ListItem::row(
        key.0.clone(),
        account.label.clone(),
        rsx! {
            Row {
                title: account.label.clone(),
                content,
                state,
                common: Common {
                    extra_class: ds::root::pass_through::ExtraClass::parse("account-row").ok(),
                    ..Common::default()
                },
                accessory: Accessory::Slot(rsx! {
                    OpenButton {
                        label: "Manage",
                        subject: account.label.clone(),
                        onpress: move |()| onopen.call(opened.clone()),
                    }
                }),
                onclick: move |_| onselect.call(picked.clone()),
            }
        },
    )
}

/// The accounts in groups, one row each, and the foot that adds and removes; the empty state
/// when there are none. A row's Manage button opens the account.
#[component]
pub(super) fn AccountList(
    pane: Pane,
    selected: Option<AccountKey>,
    adding: Adding,
    availability: Availability,
    marks: ProviderMarks,
    onselect: EventHandler<AccountKey>,
    onopen: EventHandler<AccountKey>,
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
                    title: "No accounts",
                    description: "An app that needs an account asks for it, and it appears here.",
                }
            }
            ListFoot { adding, availability, removal, on }
        };
    }
    let items = items_of(&pane, selected.as_ref(), &marks, onselect, onopen);
    rsx! {
        div { class: "accounts-list", "data-accounts": "list",
            List::<String> {
                label: "Accounts",
                items,
                style: ListStyle::Grouped,
                cursor: selected.as_ref().map(|key| key.0.clone()),
                onselect: move |id: String| onselect.call(AccountKey(id)),
                onpick: move |id: String| onopen.call(AccountKey(id)),
            }
            ListFoot { adding, availability, removal, on }
        }
    }
}

/// The bring-your-own client ids, under a disclosure of their own (the page's Advanced one holds
/// the file's keys); nothing when the account service lists none.
#[component]
pub(super) fn ClientIds(pane: Pane, onedit: EventHandler<(KeyPath, String)>) -> Element {
    let mut opened = use_signal(|| Shown::Hidden);
    if pane.clients.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "accounts-clients", "data-accounts": "clients",
            Disclosure {
                common: Common { id: Some("sign-in-clients".to_owned()), ..Common::default() },
                label: Some("Sign-in keys".to_owned()),
                shown: opened(),
                on_toggle: move |next| opened.set(next),
                FieldGroup {
                    for client in pane.clients.iter().cloned() {
                        FieldRow {
                            key: "{client.path.0}",
                            label: TextLine::from(client.label.clone()),
                            help: (!client.help.is_empty()).then(|| TextLine::from(client.help.clone())),
                            ClientField {
                                key: "{client.path.0}/{client.value}",
                                label: client.label.clone(),
                                value: client.value.clone(),
                                availability: match client.entry {
                                    ClientEntry::Open => Availability::Enabled,
                                    ClientEntry::WaitsForKey => Availability::Disabled,
                                },
                                onchange: {
                                    let path = client.path.clone();
                                    move |text| onedit.call((path.clone(), text))
                                },
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
