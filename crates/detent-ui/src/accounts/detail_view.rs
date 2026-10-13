//! One account: how it stands, its services, its sync switches (and the deletions waiting on them), the apps' permissions, Sign in again and Remove.

use super::adapter::{ActionButton, BackButton, HoldBanner};
use super::list_view::Note;
use crate::marks::ProviderMarks;
use detent_model::{
    Account, AccountKey, AccountService, AccountSync, DatasetName, HoldInput, Holds, ModuleId,
    ModuleRow, SyncHold, reauth_help,
};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_settings::schema::ActionWeight;

/// What the detail screen reports to the pane.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Handlers {
    /// Back to the list.
    pub back: EventHandler<()>,
    /// A service was switched.
    pub switched: EventHandler<(ModuleRow, bool)>,
    /// A permission's action or Sign in again was pressed.
    pub ran: EventHandler<ModuleRow>,
    /// Remove was pressed.
    pub remove: EventHandler<ModuleRow>,
    /// Sign out was pressed.
    pub sign_out: EventHandler<ModuleRow>,
    /// Keep or Remove was pressed on a held deletion.
    pub hold: EventHandler<HoldInput>,
    /// Sync now was pressed on a sync row.
    pub sync_now: EventHandler<DatasetName>,
}

fn availability_of(busy: bool) -> Availability {
    if busy {
        Availability::Disabled
    } else {
        Availability::Enabled
    }
}

/// `account`'s screen. `busy` is whether a change is in flight: its actions wait. `holds` are the
/// deletions waiting for the person's word; `marks` draw the provider.
#[component]
pub(super) fn AccountDetail(
    account: Account,
    busy: bool,
    holds: Holds,
    marks: ProviderMarks,
    on: Handlers,
) -> Element {
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
                if let Some(provider) = account.provider.clone() {
                    FieldRow { label: TextLine::from("Account type"),
                        div { class: "account-kind",
                            ProviderMark {
                                provider: marks.of(&provider.id).provider,
                                face: marks.of(&provider.id).face,
                                size: ControlSize::Small,
                            }
                            Label { text: provider.name, role: LabelRole::Secondary }
                        }
                    }
                }
                if let Some(expiry) = account.expires.clone() {
                    FieldRow {
                        label: TextLine::from(expiry.label),
                        help: Some(TextLine::from(expiry.help)),
                        Label { text: expiry.text, role: LabelRole::Secondary }
                    }
                }
                if let Some((path, name)) = account.reauth.clone() {
                    FieldRow {
                        label: TextLine::from(name.clone()),
                        help: Some(TextLine::from(reauth_help(account.sign_in))),
                        ActionButton {
                            label: name,
                            subject: account.label.clone(),
                            weight: ActionWeight::Plain,
                            availability,
                            onpress: {
                                let row = account.row(&path);
                                move |()| on.ran.call(row.clone())
                            },
                        }
                    }
                }
                if let Some((path, name)) = account.sign_out.clone() {
                    FieldRow {
                        label: TextLine::from(name.clone()),
                        ActionButton {
                            label: name,
                            subject: account.label.clone(),
                            weight: ActionWeight::Plain,
                            availability,
                            onpress: {
                                let row = account.row(&path);
                                move |()| on.sign_out.call(row.clone())
                            },
                        }
                    }
                }
            }
            if !account.services.is_empty() {
                FieldGroup { title: Some("Services".to_owned()),
                    for service in account.services.iter().cloned() {
                        ServiceRow { key: "{service.path.0}", module: account.module, service, availability, on }
                    }
                }
                Note { text: "Whether apps may use each service of the account." }
            }
            if !account.sync.is_empty() || holds.any_of(&account) {
                FieldGroup { title: Some("Sync".to_owned()),
                    for sync in account.sync.iter().cloned() {
                        SyncRow { key: "{sync.path.0}", module: account.module, sync: sync.clone(), availability, on }
                        if sync.on {
                            SyncNowRow { key: "now-{sync.path.0}", account: account.key.clone(), sync: sync.clone(), availability, on }
                        }
                        for hold in holds.on_switch(&account, &sync.path).cloned() {
                            Held { key: "{hold.dataset.0}", hold, on }
                        }
                    }
                    for hold in holds.on_no_switch(&account).cloned() {
                        Held { key: "{hold.dataset.0}", hold, on }
                    }
                }
            }
            if !account.permissions.is_empty() {
                FieldGroup { title: Some("Apps".to_owned()),
                    for permission in account.permissions.iter().cloned() {
                        FieldRow {
                            key: "{permission.path.0}",
                            label: TextLine::from(permission.label.clone()),
                            help: permission.detail().map(TextLine::from),
                            ActionButton {
                                label: permission.action.clone(),
                                subject: permission.label.clone(),
                                weight: ActionWeight::Plain,
                                availability,
                                onpress: {
                                    let row = account.row(&permission.path);
                                    move |()| on.ran.call(row.clone())
                                },
                            }
                        }
                    }
                }
                Note { text: "Revoking asks the app again the next time it needs the account." }
            }
            if !account.used_by.is_empty() {
                FieldGroup { title: Some("Apps using this identity".to_owned()),
                    for (at, app) in account.used_by.iter().cloned().enumerate() {
                        FieldRow { key: "used-{at}", label: TextLine::from("App"),
                            Label { text: app, role: LabelRole::Secondary }
                        }
                    }
                }
            }
            if let Some((path, name)) = account.remove.clone() {
                FieldGroup {
                    FieldRow {
                        label: TextLine::from(name.clone()),
                        ActionButton {
                            label: name,
                            subject: account.label.clone(),
                            weight: ActionWeight::Destructive,
                            availability,
                            onpress: {
                                let row = account.row(&path);
                                move |()| on.remove.call(row.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

/// A service's switch.
#[component]
fn ServiceRow(
    module: ModuleId,
    service: AccountService,
    availability: Availability,
    on: Handlers,
) -> Element {
    let label = service.label();
    let row = ModuleRow::new(module, service.path.clone());
    rsx! {
        FieldRow { label: TextLine::from(label.clone()), availability,
            Toggle {
                label,
                value: if service.on { Check::On } else { Check::Off },
                availability,
                onchange: move |next| on.switched.call((row.clone(), next == Check::On)),
            }
        }
    }
}

/// A sync switch, in the module's own words.
#[component]
fn SyncRow(
    module: ModuleId,
    sync: AccountSync,
    availability: Availability,
    on: Handlers,
) -> Element {
    let row = ModuleRow::new(module, sync.path.clone());
    rsx! {
        FieldRow {
            label: TextLine::from(sync.label.clone()),
            help: (!sync.help.is_empty()).then(|| TextLine::from(sync.help.clone())),
            availability,
            Toggle {
                label: sync.label.clone(),
                value: if sync.on { Check::On } else { Check::Off },
                availability,
                onchange: move |next| on.switched.call((row.clone(), next == Check::On)),
            }
        }
    }
}

/// A sync switch's "Sync now": has the sync service sync its dataset at once.
#[component]
fn SyncNowRow(
    account: AccountKey,
    sync: AccountSync,
    availability: Availability,
    on: Handlers,
) -> Element {
    let dataset = DatasetName::of_switch(&account, &sync.path);
    rsx! {
        FieldRow {
            label: TextLine::from("Sync now"),
            help: Some(TextLine::from(sync.label.clone())),
            availability,
            ActionButton {
                label: "Sync now".to_owned(),
                subject: sync.label.clone(),
                weight: ActionWeight::Plain,
                availability,
                onpress: move |()| on.sync_now.call(dataset.clone()),
            }
        }
    }
}

/// A held deletion, under the sync row it is on.
#[component]
fn Held(hold: SyncHold, on: Handlers) -> Element {
    let dataset: DatasetName = hold.dataset.clone();
    let (keep, remove) = (dataset.clone(), dataset.clone());
    rsx! {
        div { class: "accounts-hold", "data-accounts": "hold", "data-dataset": "{dataset.0}",
            HoldBanner {
                hold,
                onkeep: move |()| on.hold.call(HoldInput::Keep(keep.clone())),
                onremove: move |()| on.hold.call(HoldInput::Remove(remove.clone())),
            }
        }
    }
}
