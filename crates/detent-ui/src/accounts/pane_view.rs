//! The Accounts page's pane: the accounts, or why they are not there.

use super::adapter::RemoveAlert;
use super::detail_view::{AccountDetail, Handlers};
use super::drive::{drive, is_valued};
use super::hold_drive::drive_holds;
use super::link::AccountLink;
use super::list_view::{AccountList, ClientIds};
use crate::env::Env;
use crate::live::{Daemon, Followed, follow, follow_holds};
use crate::marks::ProviderMarks;
use detent_model::{Absence, AccountsInput, AccountsReady, AccountsState, HoldInput, Holds};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds_settings::live::LiveClient;

/// The accounts as a list and a detail screen, or one row that says the account service is not
/// there.
#[component]
pub fn AccountsPane() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(AccountsState::default);
    let client = use_signal(|| None::<LiveClient>);
    let mut set_client = client;
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

    use_future({
        let bus = env.bus.clone();
        move || {
            let bus = bus.clone();
            async move {
                follow(&bus, &Daemon::accountd(), is_valued, |followed| {
                    let run = |input| spawn(drive(state, client, bus.clone(), toasts, input));
                    match followed {
                        Followed::Opened(opened, described) => {
                            set_client.set(Some(opened));
                            run(AccountsInput::Found(Ok(described)));
                        }
                        Followed::Gone(absence) => {
                            set_client.set(None);
                            run(AccountsInput::Found(Err(absence)));
                        }
                        Followed::Changed(path, value) => {
                            run(AccountsInput::heard(path, &value));
                        }
                    }
                })
                .await;
            }
        }
    });

    let send = EventHandler::new({
        let bus = env.bus.clone();
        move |input: AccountsInput| {
            spawn(drive(state, client, bus.clone(), toasts, input));
        }
    });
    let send_hold = EventHandler::new({
        let bus = env.bus.clone();
        move |input: HoldInput| {
            spawn(drive_holds(holds, bus.clone(), toasts, input));
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
        AccountsState::Probing => rsx! {},
        AccountsState::Unavailable(absence) => rsx! { Unavailable { absence } },
        AccountsState::Ready(ready) => rsx! {
            Ready { ready, holds: holds(), marks: env.marks.clone(), send, send_hold }
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
) -> Element {
    let busy = ready.pending.is_some();
    let availability = if busy {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    let on = Handlers {
        back: EventHandler::new(move |()| send.call(AccountsInput::Back)),
        switched: EventHandler::new(move |(path, on)| {
            send.call(AccountsInput::Switched { path, on });
        }),
        ran: EventHandler::new(move |path| send.call(AccountsInput::Ran(path))),
        remove: EventHandler::new(move |path| send.call(AccountsInput::AskRemove(path))),
        sign_out: EventHandler::new(move |path| send.call(AccountsInput::AskSignOut(path))),
        hold: send_hold,
    };
    let asked = ready.confirming.clone();
    rsx! {
        div { class: "accounts", "data-accounts": "ready",
            if let Some(account) = ready.open().cloned() {
                AccountDetail { account, busy, holds, marks, on }
            } else {
                AccountList {
                    pane: ready.pane.clone(),
                    adding: ready.adding,
                    availability,
                    onopen: move |key| send.call(AccountsInput::Open(key)),
                    onadd: move |()| send.call(AccountsInput::AddAccount),
                }
                ClientIds {
                    pane: ready.pane.clone(),
                    onedit: move |(path, text)| send.call(AccountsInput::ClientEdited { path, text }),
                }
            }
            if let Some(question) = asked {
                RemoveAlert {
                    question,
                    remove_name: ready
                        .open()
                        .and_then(|account| account.remove.clone())
                        .map(|(_, name)| name)
                        .unwrap_or_default(),
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
