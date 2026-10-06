//! The list of accounts, and the sign-in client ids under it.

use super::adapter::{ClientField, OpenButton};
use detent_model::{AccountKey, AccountsPane as Pane};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::controls::disclosure::Disclosure;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::KeyPath;

/// One row per account: its name and how it stands, and the button that opens it.
#[component]
pub(super) fn AccountList(pane: Pane, onopen: EventHandler<AccountKey>) -> Element {
    if pane.accounts.is_empty() {
        return rsx! {
            div { class: "accounts-empty", "data-accounts": "empty",
                EmptyState {
                    title: "No accounts",
                    description: "An app that needs an account asks for it, and it appears here.",
                }
            }
        };
    }
    rsx! {
        div { class: "accounts-list", "data-accounts": "list",
            FieldGroup { title: Some("Accounts".to_owned()),
                for account in pane.accounts.iter().cloned() {
                    FieldRow {
                        key: "{account.key.0}",
                        label: TextLine::from(account.label.clone()),
                        help: Some(TextLine::from(account.status.label())),
                        common: Common { extra_class: ds::root::pass_through::ExtraClass::parse("account-row").ok(), ..Common::default() },
                        OpenButton {
                            label: "Manage",
                            onpress: {
                                let key = account.key.clone();
                                move |()| onopen.call(key.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

/// The bring-your-own client ids, under a disclosure of their own (the page's Advanced one holds
/// the file's keys); nothing when accountd lists none.
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
                label: Some("Sign-in clients".to_owned()),
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
