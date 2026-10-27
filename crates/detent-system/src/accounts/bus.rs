//! The account service and the key service on the bus, as the Accounts pane's link.

use super::AccountsLink;
use super::add::{add_account, add_ai_key};
use super::read::read_accounts;
use crate::bus::BusSource;
use detent_model::{Absence, AccountsPane, AddEnd, Work};

/// The services on the bus they are built over.
#[derive(Debug)]
pub struct OnBus(pub BusSource);

impl AccountsLink for OnBus {
    fn read(&self) -> Work<'_, Result<AccountsPane, Absence>> {
        Box::pin(read_accounts(&self.0))
    }

    fn add_account(&self) -> Work<'_, AddEnd> {
        Box::pin(add_account(&self.0))
    }

    fn add_ai_key(&self) -> Work<'_, AddEnd> {
        Box::pin(add_ai_key(&self.0))
    }
}
