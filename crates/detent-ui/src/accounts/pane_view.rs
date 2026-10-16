//! The Accounts page's pane: the accounts, or why they are not there.

use super::adapter::RemoveAlert;
use super::detail_view::{AccountDetail, Handlers};
use super::drive::drive;
use super::feeds::Feeds;
use super::follow_module::follow_module;
use super::hold_drive::{drive_holds, drive_sync_now};
use super::link::AccountLink;
use super::list_view::{AccountList, ClientIds, FootHandlers};
use crate::env::Env;
use crate::live::follow_holds;
use crate::marks::ProviderMarks;
use detent_model::{
    Absence, AccountsInput, AccountsReady, AccountsState, DatasetName, HoldInput, Holds, ModuleId,
    ModuleRow,
};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;

/// The accounts as a list and a detail screen, or one row that says the account service is not
/// there.
#[component]
pub fn AccountsPane() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(AccountsState::default);
    let feeds = Feeds::new();
    let holds = use_signal(Holds::default);

    // The sync service's held deletions are told apart from the accounts' rows.
    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move {
                follow_holds(&bus, |input| {
                    spawn(drive_holds(holds, bus.clone(), toasts, input));
                })
                .await;
            }
        }
    });

    // Each module is followed on its own: one that is not there leaves only its part out.
    use_future({
        let bus = env.bus.clone();
        move || follow_module(ModuleId::Accountd, bus.clone(), state, feeds, toasts)
    });
    use_future({
        let bus = env.bus.clone();
        move || follow_module(ModuleId::Inferd, bus.clone(), state, feeds, toasts)
    });
    use_future({
        let bus = env.bus.clone();
        move || follow_module(ModuleId::Syncd, bus.clone(), state, feeds, toasts)
    });

    let send = EventHandler::new({
        let bus = env.bus.clone();
        move |input: AccountsInput| {
            spawn(drive(state, feeds, bus.clone(), toasts, input));
        }
    });
    let send_hold = EventHandler::new({
        let bus = env.bus.clone();
        move |input: HoldInput| {
            spawn(drive_holds(holds, bus.clone(), toasts, input));
        }
    });

    let send_sync = EventHandler::new({
        let bus = env.bus.clone();
        move |dataset: DatasetName| {
            spawn(drive_sync_now(bus.clone(), toasts, dataset));
        }
    });

    // A link from another page asked for an account: open it once the list is there.
    let link = use_context::<AccountLink>();
    use_effect(move || {
        if let (AccountsState::Ready(ready), Some(wanted)) = (state(), link.wanted()) {
            send.call(wanted.input(&ready.pane));
            link.clear();
        }
    });

    match state() {
        AccountsState::Probing(_) => rsx! {},
        AccountsState::Unavailable(absence, _) => rsx! { Unavailable { absence } },
        AccountsState::Ready(ready) => rsx! {
            Ready { ready: *ready, holds: holds(), marks: env.marks.clone(), send, send_hold, send_sync }
        },
    }
}

#[component]
fn Ready(
    ready: AccountsReady,
    holds: Holds,
    marks: ProviderMarks,
    send: EventHandler<AccountsInput>,
    send_hold: EventHandler<HoldInput>,
    send_sync: EventHandler<DatasetName>,
) -> Element {
    let busy = ready.pending.is_some();
    let availability = if busy {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    let on = Handlers {
        back: EventHandler::new(move |()| send.call(AccountsInput::Back)),
        switched: EventHandler::new(move |(row, on)| {
            send.call(AccountsInput::Switched { row, on });
        }),
        ran: EventHandler::new(move |row| send.call(AccountsInput::Ran(row))),
        remove: EventHandler::new(move |row| send.call(AccountsInput::AskRemove(row))),
        sign_out: EventHandler::new(move |row| send.call(AccountsInput::AskSignOut(row))),
        hold: send_hold,
        sync_now: send_sync,
    };
    let asked = ready.confirming.clone();
    let remove_name = asked
        .as_ref()
        .and_then(|question| ready.pane.owner(&question.row))
        .and_then(|account| account.remove.clone())
        .map(|(_, name)| name)
        .unwrap_or_default();
    // The row the list's minus runs: the selected account's remove row.
    let removal_row = ready
        .selected
        .as_ref()
        .and_then(|key| ready.pane.account(key))
        .and_then(|account| {
            account
                .remove
                .as_ref()
                .map(|(path, _)| ModuleRow::new(account.module, path.clone()))
        });
    rsx! {
        div { class: "accounts", "data-accounts": "ready",
            if let Some(account) = ready.open().cloned() {
                AccountDetail { account, busy, holds, marks, on }
            } else {
                AccountList {
                    pane: ready.pane.clone(),
                    selected: ready.selected.clone(),
                    adding: ready.adding,
                    availability,
                    marks,
                    onselect: move |key| send.call(AccountsInput::Select(key)),
                    onopen: move |key| send.call(AccountsInput::Open(key)),
                    on: FootHandlers {
                        add: EventHandler::new(move |()| send.call(AccountsInput::AddAccount)),
                        add_key: EventHandler::new(move |()| send.call(AccountsInput::AddAiKey)),
                        continue_as: EventHandler::new(move |row| send.call(AccountsInput::Ran(row))),
                        remove: EventHandler::new(move |()| {
                            if let Some(row) = removal_row.clone() {
                                send.call(AccountsInput::AskRemove(row));
                            }
                        }),
                    },
                }
                ClientIds {
                    pane: ready.pane.clone(),
                    onedit: move |(row, text)| send.call(AccountsInput::ClientEdited { row, text }),
                    onclear: move |row| send.call(AccountsInput::SecretCleared(row)),
                }
            }
            if let Some(question) = asked {
                RemoveAlert {
                    question,
                    remove_name,
                    onanswer: move |yes| {
                        send.call(if yes { AccountsInput::Remove } else { AccountsInput::Keep });
                    },
                }
            }
        }
    }
}

/// What the absence of the account service says.
fn absence_help(absence: &Absence) -> String {
    match absence {
        Absence::NoBus => {
            "Settings is not connected to the desktop, so there are no accounts to show.".to_owned()
        }
        Absence::NotRunning => {
            "The accounts service is not running, so there are no accounts to show.".to_owned()
        }
        Absence::NotPermitted => "Accounts can only be managed when Settings is opened from \
            the desktop. Close this window and open Settings again from the launcher."
            .to_owned(),
        Absence::Unavailable => "The accounts service could not be confirmed, so there are no \
            accounts to show."
            .to_owned(),
        Absence::Failed(_) => {
            "The accounts service did not answer. Try again in a moment.".to_owned()
        }
    }
}

/// The pane's place when the account service cannot be reached: one row that says so, blocking nothing.
#[component]
fn Unavailable(absence: Absence) -> Element {
    rsx! {
        div { class: "accounts", "data-accounts": "unavailable",
            FieldGroup { title: Some("Accounts".to_owned()),
                FieldRow {
                    label: TextLine::from("Accounts"),
                    help: Some(TextLine::from(absence_help(&absence))),
                    availability: Availability::Disabled,
                    Label { text: "Unavailable", role: LabelRole::Secondary }
                }
            }
        }
    }
}
