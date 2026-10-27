//! The line a client notice is said as: the client names the fact, and the words are here.

use super::{account_notices, account_sync, computers, engines, picker, wifi};
use detent_client::Notice;

/// What the person reads for `notice`; nothing for one that is not for them to read.
pub(crate) fn notice(notice: &Notice) -> Option<String> {
    match notice {
        Notice::Words(words) => Some(words.clone()),
        Notice::ComputerRefused(refusal) => Some(computers::server_refusal(refusal)),
        Notice::AgentBlocked(reason) => Some(picker::agent_word(reason)),
        Notice::CouldNotJoin => Some(wifi::could_not_join().to_owned()),
        Notice::RestartFailed => Some(engines::restart_failed().to_owned()),
        Notice::PoolRefused => Some(picker::pool_refused().to_owned()),
        Notice::HoldNotRemoved => Some(account_sync::HOLD_NOT_REMOVED.to_owned()),
        Notice::Account(notice) => Some(account_notices::account_notice(notice)),
        Notice::AddEnded(end) => account_notices::add_end_line(end).map(str::to_owned),
        Notice::SyncNow(end) => Some(account_sync::sync_now_line(*end).to_owned()),
        Notice::Recorded(_) => None,
    }
}
