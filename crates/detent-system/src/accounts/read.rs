//! Reading the account modules once: what the intents provider answers from. Never starts a
//! daemon.

use crate::bus::{BusSource, Daemon, read_once};
use crate::deadline::{Deadline, peer};
use detent_model::{Absence, AccountsPane, ModuleId, Sources, accounts_row_is_valued};
use ds_peer::within;

/// The accounts the modules list now, read once. A module that is not there leaves its part out;
/// the answer is an absence only when none is there.
///
/// The whole read is under one action deadline besides each call's own: three modules of a
/// row's worth of calls each must not hold an agent's request for as long as every call could.
pub async fn read_accounts(bus: &BusSource) -> Result<AccountsPane, Absence> {
    within(Deadline::ACTION, peer::ACCOUNTS, read_modules(bus))
        .await
        .map_err(|_| Absence::Failed)?
}

async fn read_modules(bus: &BusSource) -> Result<AccountsPane, Absence> {
    let mut sources = Sources::default();
    for module in ModuleId::ALL {
        let found = read_once(bus, &Daemon::of(module), accounts_row_is_valued).await;
        sources.hear(module, found);
    }
    match sources.pane() {
        Some(pane) => Ok(pane),
        None => Err(sources.absence().unwrap_or(Absence::NotRunning)),
    }
}
