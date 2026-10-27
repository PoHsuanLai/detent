//! The accounts, as the intents provider asks for them: read from accountd over the session bus.

use detent_intents::{AccountsRead, AccountsSource};
use detent_system::accounts::read_accounts;
use detent_system::bus::BusSource;

/// Reads accountd's accounts each time it is asked.
#[derive(Debug, Clone)]
pub struct LiveAccounts(pub BusSource);

impl AccountsSource for LiveAccounts {
    fn read(&self) -> AccountsRead<'_> {
        Box::pin(read_accounts(&self.0))
    }
}
