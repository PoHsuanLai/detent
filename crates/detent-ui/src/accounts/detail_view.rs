//! One account: who it is and how it signs in, how it stands, its services (with what the sync
//! service says of each), this month's spend, its sync switches (and the deletions waiting on
//! them), the apps with access, Sign in again and Remove.

use super::adapter::{ActionButton, HoldBanner};
use super::attention_view::AttentionBanner;
use super::avatar_view::{AccountAvatar, AvatarPlace};
use super::grants_view::AppsWithAccess;
use super::list_view::Note;
use super::switch::{check_of, switch_of};
use crate::copy::account_sync::{spend_line, sync_lines};
use crate::copy::accounts::{reauth_help, service_label, sign_in_words, status_label};
use crate::marks::ProviderMarks;
use detent_model::{
    Account, AccountKey, AccountService, AccountSync, Attention, DatasetName, HoldInput, Holds,
    ModuleId, ModuleRow, SwitchState, SyncBook, SyncHold,
};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::fields::field_help::HelpShown;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::ActionWeight;

/// What the detail screen reports to the pane.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Handlers {
    /// A service was switched.
    pub switched: EventHandler<(ModuleRow, SwitchState)>,
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

/// `account`'s screen. `availability` is `Disabled` while a change is in flight: its actions
/// wait. `holds` are the deletions waiting for the person's word; `marks` draw the provider;
/// `book` is what the sync service says of the account's datasets.
#[component]
pub(super) fn AccountDetail(
    account: Account,
    availability: Availability,
    holds: Holds,
    marks: ProviderMarks,
    book: SyncBook,
    on: Handlers,
) -> Element {
    let attention = Attention::of(&account);
    rsx! {
        div { class: "accounts-detail", "data-accounts": "detail", "data-account": "{account.key.0}",
            if let Some(attention) = attention.clone() {
                AttentionBanner {
                    account: account.clone(),
                    attention,
                    availability,
                    onsign_in: move |row| on.ran.call(row),
                }
            }
            Header { account: account.clone(), marks, book: book.clone() }
            FieldGroup {
                FieldRow {
                    label: TextLine::from(crate::copy::account_views::STATUS),
                    Label { text: status_label(&account.status), role: LabelRole::Secondary }
                }
                if let (None, Some(expiry)) = (&attention, account.expires.clone()) {
                    FieldRow {
                        label: TextLine::from(expiry.label),
                        help: Some(TextLine::from(expiry.help)),
                        Label { text: expiry.text, role: LabelRole::Secondary }
                    }
                }
                if let (None, Some((path, name))) = (&attention, account.reauth.clone()) {
                    FieldRow {
                        label: TextLine::from(name.clone()),
                        help: Some(TextLine::from(reauth_help(account.sign_in))),
                        help_shown: HelpShown::OnRequest,
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
                FieldGroup { title: Some(crate::copy::account_views::SERVICES.to_owned()),
                    for service in account.services.iter().cloned() {
                        ServiceRow {
                            key: "{service.path.0}",
                            module: account.module,
                            line: service
                                .dataset(&account.key)
                                .and_then(|dataset| book.status_of(&dataset).map(sync_lines)),
                            service,
                            availability,
                            on,
                        }
                    }
                }
                Note { text: crate::copy::account_views::SERVICES_HELP }
            }
            if let Some(bar) = account.spend.bar() {
                FieldGroup { title: Some(crate::copy::account_views::SPENDING.to_owned()),
                    FieldRow {
                        label: TextLine::from(crate::copy::account_views::THIS_MONTH),
                        help: Some(TextLine::from(spend_line(bar.line))),
                        ProgressIndicator {
                            style: ProgressStyle::Bar,
                            progress: Progress::Known(Fraction(bar.permille)),
                            common: Common { aria_label: Some(crate::copy::account_views::SPENT_THIS_MONTH.to_owned()), ..Common::default() },
                        }
                    }
                }
            }
            if !account.sync.is_empty() || account.show_in_files.is_some() || holds.any_of(&account) {
                FieldGroup { title: Some(crate::copy::account_views::SYNC.to_owned()),
                    if let Some(files) = account.show_in_files.clone() {
                        ShowInFilesRow { module: account.module, files, availability, on }
                    }
                    for sync in account.sync.iter().cloned() {
                        SyncRow { key: "{sync.path.0}", module: account.module, sync: sync.clone(), availability, on }
                        if sync.on.is_on() {
                            SyncNowRow {
                                key: "now-{sync.path.0}",
                                account: account.key.clone(),
                                line: book
                                    .status_of(&DatasetName::of_switch(&account.key, &sync.path))
                                    .map(sync_lines),
                                sync: sync.clone(),
                                availability,
                                on,
                            }
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
                SectionHeader { title: crate::copy::account_views::APPS_WITH_ACCESS.to_owned() }
                AppsWithAccess {
                    account: account.clone(),
                    availability,
                    onrevoke: move |row| on.ran.call(row),
                }
                Note { text: crate::copy::account_views::REVOKE_HELP }
            }
            if !account.used_by.is_empty() {
                FieldGroup { title: Some(crate::copy::account_views::APPS_USING_THIS_IDENTITY.to_owned()),
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

/// The head of the screen: the account's avatar, its name, its kind and how it signs in.
#[component]
fn Header(account: Account, marks: ProviderMarks, book: SyncBook) -> Element {
    let kind = account
        .provider
        .as_ref()
        .map(|provider| provider.name.clone());
    rsx! {
        div { class: "accounts-header",
            AccountAvatar {
                label: account.label.clone(),
                provider: account.provider.clone(),
                marks,
                place: AvatarPlace::Header,
                work: book.work_of(&account),
            }
            div { class: "accounts-header-words",
                Label { text: account.label.clone(), role: LabelRole::Primary, style: LabelStyle::Title }
                if let Some(kind) = kind {
                    Label { text: kind, role: LabelRole::Secondary }
                }
                if let Some(how) = sign_in_words(account.sign_in) {
                    Label { text: how, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
        }
    }
}

/// A service's switch, with the sync service's words under it when it keeps the service's data.
#[component]
fn ServiceRow(
    module: ModuleId,
    service: AccountService,
    line: Option<String>,
    availability: Availability,
    on: Handlers,
) -> Element {
    let label = service_label(&service);
    let row = ModuleRow::new(module, service.path.clone());
    rsx! {
        FieldRow {
            label: TextLine::from(label.clone()),
            help: line.map(TextLine::from),
            availability,
            Toggle {
                label,
                value: check_of(service.on),
                availability,
                onchange: move |next| on.switched.call((row.clone(), switch_of(next))),
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
            help_shown: HelpShown::OnRequest,
            availability,
            Toggle {
                label: sync.label.clone(),
                value: check_of(sync.on),
                availability,
                onchange: move |next| on.switched.call((row.clone(), switch_of(next))),
            }
        }
    }
}

/// "Show in Files": the account's whole drive in Files. Its line stands under it, so the service's
/// word after a turn-on that did not end on ("Files wasn't turned on. You can try again.") is read
/// without opening anything.
#[component]
fn ShowInFilesRow(
    module: ModuleId,
    files: AccountSync,
    availability: Availability,
    on: Handlers,
) -> Element {
    let row = ModuleRow::new(module, files.path.clone());
    rsx! {
        FieldRow {
            label: TextLine::from(files.label.clone()),
            help: (!files.help.is_empty()).then(|| TextLine::from(files.help.clone())),
            help_shown: HelpShown::Inline,
            availability,
            Toggle {
                label: files.label.clone(),
                value: check_of(files.on),
                availability,
                onchange: move |next| on.switched.call((row.clone(), switch_of(next))),
            }
        }
    }
}

/// A sync switch's "Sync now": has the sync service sync its dataset at once.
#[component]
fn SyncNowRow(
    account: AccountKey,
    sync: AccountSync,
    line: Option<String>,
    availability: Availability,
    on: Handlers,
) -> Element {
    let dataset = DatasetName::of_switch(&account, &sync.path);
    // The sync service's words ("Synced 5 minutes ago. 2.1 GB of 15 GB") stand under the row when
    // it has them; the switch's name waits behind the info button otherwise.
    let (help, shown) = match line {
        Some(words) => (words, HelpShown::Inline),
        None => (sync.label.clone(), HelpShown::OnRequest),
    };
    rsx! {
        FieldRow {
            label: TextLine::from(crate::copy::account_views::SYNC_NOW),
            help: Some(TextLine::from(help)),
            help_shown: shown,
            availability,
            ActionButton {
                label: crate::copy::account_views::SYNC_NOW.to_owned(),
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
