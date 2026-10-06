//! The Accounts page's pane: accountd's accounts, or why they are not there.

use super::adapter::RemoveAlert;
use super::detail_view::{AccountDetail, Handlers};
use super::drive::{drive, is_valued};
use super::link::AccountLink;
use super::list_view::{AccountList, ClientIds};
use crate::env::Env;
use crate::live::{Daemon, Followed, follow};
use detent_model::{Absence, AccountsInput, AccountsReady, AccountsState};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds_settings::live::LiveClient;

/// accountd's accounts as a list and a detail screen, or one row that says accountd is not
/// there.
#[component]
pub fn AccountsPane() -> Element {
    let env = use_context::<Env>();
    let toasts = use_toasts();
    let state = use_signal(AccountsState::default);
    let mut client = use_signal(|| None::<LiveClient>);

    use_future(move || {
        let bus = env.bus.clone();
        async move {
            follow(
                &bus,
                &Daemon::accountd(),
                is_valued,
                |followed| match followed {
                    Followed::Opened(opened, described) => {
                        client.set(Some(opened));
                        spawn(drive(
                            state,
                            client,
                            toasts,
                            AccountsInput::Found(Ok(described)),
                        ));
                    }
                    Followed::Gone(absence) => {
                        client.set(None);
                        spawn(drive(
                            state,
                            client,
                            toasts,
                            AccountsInput::Found(Err(absence)),
                        ));
                    }
                    Followed::Changed(..) => {
                        spawn(drive(state, client, toasts, AccountsInput::Changed));
                    }
                },
            )
            .await;
        }
    });

    let send = EventHandler::new(move |input: AccountsInput| {
        spawn(drive(state, client, toasts, input));
    });

    // A link from another page asked for an account: open it once the list is there.
    let link = use_context::<AccountLink>();
    use_effect(move || {
        if let (AccountsState::Ready(_), Some(key)) = (state(), link.wanted()) {
            send.call(AccountsInput::Open(key));
            link.clear();
        }
    });

    match state() {
        AccountsState::Probing => rsx! {},
        AccountsState::Unavailable(absence) => rsx! { Unavailable { absence } },
        AccountsState::Ready(ready) => rsx! { Ready { ready, send } },
    }
}

#[component]
fn Ready(ready: AccountsReady, send: EventHandler<AccountsInput>) -> Element {
    let busy = ready.pending.is_some();
    let on = Handlers {
        back: EventHandler::new(move |()| send.call(AccountsInput::Back)),
        switched: EventHandler::new(move |(path, on)| {
            send.call(AccountsInput::Switched { path, on });
        }),
        ran: EventHandler::new(move |path| send.call(AccountsInput::Ran(path))),
        remove: EventHandler::new(move |path| send.call(AccountsInput::AskRemove(path))),
    };
    let removal = ready
        .confirming
        .clone()
        .zip(ready.open().and_then(|account| account.remove.clone()));
    rsx! {
        div { class: "accounts", "data-accounts": "ready",
            if let Some(account) = ready.open().cloned() {
                AccountDetail { account, busy, on }
            } else {
                AccountList {
                    pane: ready.pane.clone(),
                    onopen: move |key| send.call(AccountsInput::Open(key)),
                }
                ClientIds {
                    pane: ready.pane.clone(),
                    onedit: move |(path, text)| send.call(AccountsInput::ClientEdited { path, text }),
                }
            }
            if let Some((question, (_, name))) = removal {
                RemoveAlert {
                    question,
                    action: name,
                    onanswer: move |yes| {
                        send.call(if yes { AccountsInput::Remove } else { AccountsInput::Keep });
                    },
                }
            }
        }
    }
}

/// What the absence of accountd says.
fn absence_help(absence: &Absence) -> String {
    match absence {
        Absence::NoBus => "Settings is not connected to the session bus.".to_owned(),
        Absence::NotRunning => {
            "accountd is not running, so there are no accounts to show.".to_owned()
        }
        Absence::NotPermitted => {
            "accountd does not list Settings among its callers (callers.toml).".to_owned()
        }
        Absence::Failed(why) => format!("accountd could not be asked: {why}"),
    }
}

/// The pane's place when accountd cannot be reached: one row that says so, blocking nothing.
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
