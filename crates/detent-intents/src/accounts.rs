//! Where the provider learns which accounts there are: from accountd, through the binary.

use detent_model::{Absence, AccountsPane, Work};

/// What reading the accounts is, once: a future that answers with the accounts or why not.
pub type AccountsRead<'a> = Work<'a, Result<AccountsPane, Absence>>;

/// The accounts accountd lists, as the provider asks for them. The binary reads them over the
/// session bus; a test answers from a list.
pub trait AccountsSource: Send + Sync {
    /// The accounts now; never starts accountd.
    fn read(&self) -> AccountsRead<'_>;
}
