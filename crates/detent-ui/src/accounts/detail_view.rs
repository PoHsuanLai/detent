//! One account: how it stands, its services, its sync switches, the apps' permissions, Sign in again and Remove.

use super::adapter::{ActionButton, BackButton};
use super::list_view::Note;
use detent_model::{Account, AccountService, AccountSync};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds_settings::schema::{ActionWeight, KeyPath};

/// What the detail screen reports to the pane.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Handlers {
    /// Back to the list.
    pub back: EventHandler<()>,
    /// A service was switched.
    pub switched: EventHandler<(KeyPath, bool)>,
    /// A permission's action or Sign in again was pressed.
    pub ran: EventHandler<KeyPath>,
    /// Remove was pressed.
    pub remove: EventHandler<KeyPath>,
    /// Sign out was pressed.
    pub sign_out: EventHandler<KeyPath>,
}

fn availability_of(busy: bool) -> Availability {
    if busy {
        Availability::Disabled
    } else {
        Availability::Enabled
    }
}

/// `account`'s screen. `busy` is whether a change is in flight: its actions wait.
#[component]
pub(super) fn AccountDetail(account: Account, busy: bool, on: Handlers) -> Element {
    let availability = availability_of(busy);
    rsx! {
        div { class: "accounts-detail", "data-accounts": "detail", "data-account": "{account.key.0}",
            div { class: "accounts-back",
                BackButton { label: "Accounts", onpress: move |()| on.back.call(()) }
            }
            FieldGroup { title: Some(account.label.clone()),
                FieldRow {
                    label: TextLine::from("Status"),
                    Label { text: account.status.label(), role: LabelRole::Secondary }
                }
                if let Some((path, name)) = account.reauth.clone() {
                    FieldRow {
                        label: TextLine::from(name.clone()),
                        help: Some(TextLine::from("Sign in to the account again in your browser.")),
                        ActionButton {
                            label: name,
                            weight: ActionWeight::Plain,
                            availability,
                            onpress: move |()| on.ran.call(path.clone()),
                        }
                    }
                }
                if let Some((path, name)) = account.sign_out.clone() {
                    FieldRow {
                        label: TextLine::from(name.clone()),
                        ActionButton {
                            label: name,
                            weight: ActionWeight::Plain,
                            availability,
                            onpress: move |()| on.sign_out.call(path.clone()),
                        }
                    }
                }
            }
            if !account.services.is_empty() {
                FieldGroup { title: Some("Services".to_owned()),
                    for service in account.services.iter().cloned() {
                        ServiceRow { key: "{service.path.0}", service, availability, on }
                    }
                }
                Note { text: "Whether apps may use each service of the account." }
            }
            if !account.sync.is_empty() {
                FieldGroup { title: Some("Sync".to_owned()),
                    for sync in account.sync.iter().cloned() {
                        SyncRow { key: "{sync.path.0}", sync, availability, on }
                    }
                }
            }
            if !account.permissions.is_empty() {
                FieldGroup { title: Some("Apps".to_owned()),
                    for permission in account.permissions.iter().cloned() {
                        FieldRow {
                            key: "{permission.path.0}",
                            label: TextLine::from(permission.label.clone()),
                            ActionButton {
                                label: permission.action.clone(),
                                weight: ActionWeight::Plain,
                                availability,
                                onpress: {
                                    let path = permission.path.clone();
                                    move |()| on.ran.call(path.clone())
                                },
                            }
                        }
                    }
                }
                Note { text: "Revoking asks the app again the next time it needs the account." }
            }
            if let Some((path, name)) = account.remove.clone() {
                FieldGroup {
                    FieldRow {
                        label: TextLine::from(name.clone()),
                        ActionButton {
                            label: name,
                            weight: ActionWeight::Destructive,
                            availability,
                            onpress: move |()| on.remove.call(path.clone()),
                        }
                    }
                }
            }
        }
    }
}

/// A service's switch.
#[component]
fn ServiceRow(service: AccountService, availability: Availability, on: Handlers) -> Element {
    let label = service.label();
    let path = service.path.clone();
    rsx! {
        FieldRow { label: TextLine::from(label.clone()), availability,
            Toggle {
                label,
                value: if service.on { Check::On } else { Check::Off },
                availability,
                onchange: move |next| on.switched.call((path.clone(), next == Check::On)),
            }
        }
    }
}

/// A sync switch, in the module's own words.
#[component]
fn SyncRow(sync: AccountSync, availability: Availability, on: Handlers) -> Element {
    let path = sync.path.clone();
    rsx! {
        FieldRow { label: TextLine::from(sync.label.clone()), availability,
            Toggle {
                label: sync.label.clone(),
                value: if sync.on { Check::On } else { Check::Off },
                availability,
                onchange: move |next| on.switched.call((path.clone(), next == Check::On)),
            }
        }
    }
}
