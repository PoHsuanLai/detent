//! Where the provider learns which accounts there are: from accountd, through the binary.

use detent_client::Client;
use detent_model::{Absence, Account, AccountStatus, AccountsPane, Work, humanize};

/// What reading the accounts is, once: a future that answers with the accounts or why not.
pub type AccountsRead<'a> = Work<'a, Result<AccountsPane, Absence>>;

/// The accounts accountd lists, as the provider asks for them. The binary reads them over the
/// session bus; a test answers from a list.
pub trait AccountsSource: Send + Sync {
    /// The accounts now; never starts accountd.
    fn read(&self) -> AccountsRead<'_>;
}

/// The window's own client is the source: there is one way to reach accountd, and it is the
/// window's.
impl AccountsSource for Client {
    fn read(&self) -> AccountsRead<'_> {
        self.read_accounts()
    }
}

/// The account as one line for an agent: its label, the kinds of service it offers and how it
/// stands. No id and no permission is in it.
pub(crate) fn account_line(account: &Account) -> String {
    let kinds: Vec<&str> = account.services.iter().map(|s| s.kind.as_str()).collect();
    let offers = if kinds.is_empty() {
        String::new()
    } else {
        format!(" ({})", kinds.join(", "))
    };
    format!("{}{offers}: {}", account.label, standing(&account.status))
}

/// How the account stands, in the words an agent is told.
fn standing(status: &AccountStatus) -> String {
    match status {
        AccountStatus::Ok => "Signed in".to_owned(),
        AccountStatus::NeedsReauth | AccountStatus::NeedsLogin => "Needs sign-in".to_owned(),
        AccountStatus::Offline => "Offline".to_owned(),
        AccountStatus::Limited => "Limited".to_owned(),
        AccountStatus::Other(word) => humanize(word),
    }
}
