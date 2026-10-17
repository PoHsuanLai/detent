//! The owner check quire's live client asks, answered from the desktop's caller table.

use crate::allow::allowing;
use crate::services::{SERVICE_PROOF, required_role};
use ds_settings::live::{OwnerAnswer, OwnerCheck, SenderName, Verdict};
use peephole_bus::BusPeers;
use std::fmt;
use std::sync::Arc;

/// Whether the owner of a service's bus name holds the role of that service, at the proof a unit
/// gives. A name that is not one of the seven services, an owner that cannot be identified and an
/// owner that holds another role are all a refusal.
#[derive(Clone)]
pub struct RoleCheck {
    peers: Arc<BusPeers>,
}

impl RoleCheck {
    /// The check over `peers`.
    pub fn new(peers: BusPeers) -> RoleCheck {
        RoleCheck {
            peers: Arc::new(peers),
        }
    }
}

impl fmt::Debug for RoleCheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RoleCheck").finish_non_exhaustive()
    }
}

impl OwnerCheck for RoleCheck {
    fn holds_role<'a>(&'a self, name: &'a str, owner: &'a SenderName) -> OwnerAnswer<'a> {
        Box::pin(async move {
            let Some(role) = required_role(name) else {
                return Verdict::Refuse;
            };
            let need = allowing(role, SERVICE_PROOF);
            match self.peers.require(Some(owner.0.as_str()), &need).await {
                Ok(_) => Verdict::Allow,
                Err(_) => Verdict::Refuse,
            }
        })
    }
}
