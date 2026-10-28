//! The account service from memory: the accounts they list, and how each add sheet ends.

use super::AccountsLink;
use crate::fake_kit::Kit;
use crate::lock::locked;
use detent_model::{Absence, AccountsPane, AddEnd, Work};
use std::sync::{Arc, Mutex};

/// Which sheet was opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetAsked {
    /// The add-account sheet.
    Account,
}

/// Services that answer from what they were given.
#[derive(Debug, Clone)]
pub struct FakeAccounts {
    kit: Arc<Kit<AccountsPane, SheetAsked>>,
    end: Arc<Mutex<AddEnd>>,
}

impl FakeAccounts {
    /// Services listing `pane`; every sheet ends as `AddEnd::Closed` until told otherwise.
    pub fn answering(pane: AccountsPane) -> FakeAccounts {
        FakeAccounts {
            kit: Kit::new(Ok(pane)),
            end: Arc::new(Mutex::new(AddEnd::Closed)),
        }
    }

    /// No services, for this reason.
    pub fn missing(why: Absence) -> FakeAccounts {
        FakeAccounts {
            kit: Kit::new(Err(why)),
            end: Arc::new(Mutex::new(AddEnd::Closed)),
        }
    }

    /// The sheets end like this from now on.
    pub fn sheets_end(&self, end: AddEnd) {
        *locked(&self.end) = end;
    }

    /// Which sheets were opened, in order.
    pub fn asked(&self) -> Vec<SheetAsked> {
        self.kit.asked()
    }

    fn open(&self, sheet: SheetAsked) -> Work<'_, AddEnd> {
        self.kit.record(sheet);
        let end = locked(&self.end).clone();
        Box::pin(std::future::ready(end))
    }
}

impl AccountsLink for FakeAccounts {
    fn read(&self) -> Work<'_, Result<AccountsPane, Absence>> {
        Box::pin(std::future::ready(self.kit.standing()))
    }

    fn add_account(&self) -> Work<'_, AddEnd> {
        self.open(SheetAsked::Account)
    }
}
