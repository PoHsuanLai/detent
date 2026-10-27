//! What the client holds of each module it follows: the client its rows are set and run through,
//! and which of its reads run. Each module has its own, so one that is slow or gone holds nothing
//! up. Reading again goes one read at a time: the latest ask wins.

use crate::follow::Slot;
use detent_model::{Absence, Described, ModuleId, ReadTurn, Reads, accounts_row_is_valued};
use detent_system::modules::{Module, reread};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

/// How long a failed read waits before the next.
const RETRY_AFTER: Duration = Duration::from_secs(1);

/// One module's client (none while it is not there) and its reads.
#[derive(Debug, Clone, Default)]
pub(crate) struct Feed {
    /// The client of the module, while it is on the bus.
    pub module: Slot,
    reads: Arc<Mutex<Reads>>,
}

impl Feed {
    /// Moves the reads on by `turn`, which says how, and what turn that gives.
    pub(super) fn turn(&self, turn: fn(Reads) -> (Reads, ReadTurn)) -> ReadTurn {
        let mut reads = self.reads.lock().unwrap_or_else(PoisonError::into_inner);
        let (next, turn) = turn(*reads);
        *reads = next;
        turn
    }

    /// The module read again, or `None` when another read is running: that one reads once more
    /// after it ends and answers for both. A result that something asked past is dropped for it.
    pub(crate) async fn reload(&self, opened: &Module) -> Option<Result<Described, Absence>> {
        if self.turn(Reads::ask) == ReadTurn::Rest {
            return None;
        }
        loop {
            let found = reread(opened, accounts_row_is_valued).await;
            if self.turn(Reads::finish) == ReadTurn::Rest {
                return Some(found);
            }
        }
    }

    /// [`Feed::reload`] after a wait.
    pub(crate) async fn reload_later(&self, opened: &Module) -> Option<Result<Described, Absence>> {
        tokio::time::sleep(RETRY_AFTER).await;
        self.reload(opened).await
    }
}

/// The three modules' feeds.
#[derive(Debug, Clone, Default)]
pub(crate) struct Feeds {
    accountd: Feed,
    inferd: Feed,
    syncd: Feed,
}

impl Feeds {
    /// The feed of `module`.
    pub(crate) fn of(&self, module: ModuleId) -> &Feed {
        match module {
            ModuleId::Accountd => &self.accountd,
            ModuleId::Inferd => &self.inferd,
            ModuleId::Syncd => &self.syncd,
        }
    }
}
