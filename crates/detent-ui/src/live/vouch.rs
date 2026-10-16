//! Who vouches that the owner of a service's bus name is that service.

use ds_settings::live::{OwnerAnswer, OwnerCheck, SenderName};
use std::sync::Arc;

/// An owner check a bus is held to, shared by every call made on it. The session bus needs none
/// named: it uses the desktop's caller table. A bus a test builds names one (`Vouch::of`).
#[derive(Debug, Clone)]
pub struct Vouch(Arc<dyn OwnerCheck>);

impl Vouch {
    /// The check `check`.
    pub fn of(check: impl OwnerCheck) -> Vouch {
        Vouch(Arc::new(check))
    }
}

impl OwnerCheck for Vouch {
    fn holds_role<'a>(&'a self, name: &'a str, owner: &'a SenderName) -> OwnerAnswer<'a> {
        self.0.holds_role(name, owner)
    }
}
