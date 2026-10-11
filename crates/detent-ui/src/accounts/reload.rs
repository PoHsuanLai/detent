//! Reading accountd again one read at a time: the latest ask wins.

use super::drive::is_valued;
use crate::live::reread;
use detent_model::{Absence, Described, ReadTurn, Reads};
use dioxus::prelude::*;
use ds_settings::live::LiveClient;
use std::time::Duration;

/// How long a failed read waits before the next.
const RETRY_AFTER: Duration = Duration::from_secs(1);

/// The module read again, or `None` when another read is running: that one reads once more
/// after it ends and answers for both. A result that something asked past is dropped for it.
pub(crate) async fn reload(
    mut reads: Signal<Reads>,
    opened: &LiveClient,
) -> Option<Result<Described, Absence>> {
    let (next, turn) = reads.peek().ask();
    reads.set(next);
    if turn == ReadTurn::Rest {
        return None;
    }
    loop {
        let found = reread(opened, is_valued).await;
        let (next, turn) = reads.peek().finish();
        reads.set(next);
        if turn == ReadTurn::Rest {
            return Some(found);
        }
    }
}

/// [`reload`] after a wait.
pub(crate) async fn reload_later(
    reads: Signal<Reads>,
    opened: &LiveClient,
) -> Option<Result<Described, Absence>> {
    tokio::time::sleep(RETRY_AFTER).await;
    reload(reads, opened).await
}
