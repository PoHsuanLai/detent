//! The Accounts page's pane: the accounts, or why they are not there.

use super::adapter::RemoveAlert;
use super::detail_view::{AccountDetail, Handlers};
use super::link::AccountLink;
use super::list_view::{AccountList, ClientIds, FootHandlers};
use super::sync_read::use_sync_reading;
use crate::copy::absence::{Subject, absence as absence_words};
use crate::env::Env;
use crate::live::{use_events, use_live};
use crate::marks::ProviderMarks;
use crate::system::Absent;
use detent_client::{Client, Pane};
use detent_model::{
    Absence, AccountsInput, AccountsReady, AccountsState, DatasetName, HoldInput, Holds, ModuleRow,
    SyncBook,
};
use dioxus::prelude::*;
use ds::components::chrome::split_view::model::{Collapsing, PaneSize, PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::components::overlays::empty_state::EmptyState;
use ds::prelude::*;

/// The list's widths beside the detail: 280 to start, held between 220 and 380, never folded.
const SOURCES: PaneSpec = PaneSpec {
    preferred: PaneSize::Fixed(Px(280.0)),
    min: PaneSize::Fixed(Px(220.0)),
    max: PaneSize::Fixed(Px(380.0)),
    collapsing: Collapsing::Never,
};

/// The accounts as a list and a detail screen, or one row that says the account service is not
/// there.
#[component]
pub fn AccountsPane() -> Element {
    let env = use_context::<Env>();
    let accounts = use_live(Client::accounts);
    let state = accounts.state;

    // The sync service's held deletions are told apart from the accounts' rows.
    let held = accounts.pane.holds().clone();
    let holds = use_events(held.live());
    use_future({
        let held = held.clone();
        move || {
            let held = held.clone();
            async move { held.watch().await }
        }
    });

    // What the sync service says of the accounts' datasets, read again once "Sync now" was asked.
    let datasets = accounts.pane.datasets().clone();
    let book = use_events(datasets.live());
    use_sync_reading(datasets.clone(), state);

    let send = EventHandler::new({
        let pane = accounts.pane.clone();
        move |input: AccountsInput| {
            spawn(pane.send(input));
        }
    });
    let send_hold = EventHandler::new(move |input: HoldInput| {
        spawn(held.send(input));
    });
    let send_sync = EventHandler::new(move |dataset: DatasetName| {
        let datasets = datasets.clone();
        spawn(async move { datasets.sync_now(dataset).await });
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
            Ready { ready: *ready, holds: holds(), marks: env.marks.clone(), book: book(), send, send_hold, send_sync }
        },
    }
}

#[component]
fn Ready(
    ready: AccountsReady,
    holds: Holds,
    marks: ProviderMarks,
    book: SyncBook,
    send: EventHandler<AccountsInput>,
    send_hold: EventHandler<HoldInput>,
    send_sync: EventHandler<DatasetName>,
) -> Element {
    let availability = if ready.pending.is_some() {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    let on = Handlers {
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
    let list = rsx! {
        div { class: "accounts-sources",
            AccountList {
                pane: ready.pane.clone(),
                selected: ready.selected.clone(),
                sheet_open: ready.sheet_open,
                availability,
                marks: marks.clone(),
                book: book.clone(),
                onselect: move |key| send.call(AccountsInput::Select(key)),
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
        }
    };
    rsx! {
        div { class: "accounts", "data-accounts": "ready",
            div { class: "accounts-split",
                SplitView {
                    label: crate::copy::common::ACCOUNTS,
                    panes: vec![SplitPane::new(SOURCES, list)],
                    div { class: "accounts-rest",
                        if let Some(account) = ready.open().cloned() {
                            AccountDetail { account, availability, holds, marks, book, on }
                        } else if !ready.pane.accounts.is_empty() {
                            EmptyState {
                                title: crate::copy::account_views::NO_ACCOUNT_CHOSEN,
                                description: crate::copy::account_views::CHOOSE_ACCOUNT_HINT,
                            }
                        }
                    }
                }
            }
            ClientIds {
                pane: ready.pane.clone(),
                onedit: move |(row, text)| send.call(AccountsInput::ClientEdited { row, text }),
                onclear: move |row| send.call(AccountsInput::SecretCleared(row)),
            }
            if let Some(question) = asked {
                RemoveAlert {
                    question,
                    remove_name,
                    onkeep: move |()| send.call(AccountsInput::Keep),
                    onremove: move |()| send.call(AccountsInput::Remove),
                }
            }
        }
    }
}

/// The pane's place when the account service cannot be reached: one row that says so, blocking nothing.
#[component]
fn Unavailable(absence: Absence) -> Element {
    rsx! {
        div { class: "accounts", "data-accounts": "unavailable",
            Absent {
                words: absence_words(Subject::Accounts, absence),
            }
        }
    }
}
