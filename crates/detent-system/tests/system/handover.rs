//! An owner check that vouches for everyone and, once armed, hands a service's bus name from one
//! program to another at the moment it is asked: after the owner was found and before the call.
//! A call made by name from then on reaches the second program; a call made to the owner that was
//! found reaches the first.

use ds_settings::live::{OwnerAnswer, OwnerCheck, SenderName, Verdict};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use zbus::Connection;

/// The check, and the switch that arms it. It hands the name over once, the first time it is asked
/// about that name after being armed.
#[derive(Debug)]
pub struct HandsOver {
    name: &'static str,
    armed: Arc<AtomicBool>,
    from: Connection,
    to: Connection,
}

impl HandsOver {
    /// A check that passes `name` from `from` to `to` when armed, and the switch that arms it.
    pub fn new(
        name: &'static str,
        from: Connection,
        to: Connection,
    ) -> (HandsOver, Arc<AtomicBool>) {
        let armed = Arc::new(AtomicBool::new(false));
        let check = HandsOver {
            name,
            armed: Arc::clone(&armed),
            from,
            to,
        };
        (check, armed)
    }
}

impl OwnerCheck for HandsOver {
    fn holds_role<'a>(&'a self, name: &'a str, _owner: &'a SenderName) -> OwnerAnswer<'a> {
        Box::pin(async move {
            if name == self.name && self.armed.swap(false, Ordering::SeqCst) {
                self.from.release_name(self.name).await.expect("released");
                self.to.request_name(self.name).await.expect("taken");
            }
            Verdict::Allow
        })
    }
}
