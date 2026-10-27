//! The sync service's words under an account: how a dataset stands, what a held deletion says,
//! how "Sync now" was taken, and what an AI account has spent.

use super::engine_install::size_words;
use super::updates::elapsed_words;
use detent_model::{
    Cents, HoldCounts, SpendLine, StorageQuota, SyncHold, SyncNowEnd, SyncPause, SyncStatus,
    SyncWork,
};

/// What the banner's Keep button carries as its help.
pub(crate) const KEEP_HELP: &str =
    "Keep the files here. You will be asked again if they are still gone.";

/// What the person is told when the sync service would not remove the held files.
pub(crate) const HOLD_NOT_REMOVED: &str = "Could not remove the files. Try again later.";

/// The sentence on a held deletion's banner.
pub(crate) fn hold_banner(hold: &SyncHold) -> String {
    let HoldCounts { discard, held } = hold.counts;
    if discard == held {
        format!("All {held} files were removed on the server.")
    } else {
        format!("{discard} of {held} files were removed on the server. Remove them here too?")
    }
}

/// The line the person reads for how the sync service took "Sync now".
pub(crate) fn sync_now_line(end: SyncNowEnd) -> &'static str {
    match end {
        SyncNowEnd::Started => "Syncing now.",
        SyncNowEnd::Paused => "Syncing is paused for this account.",
        SyncNowEnd::Failed => "Could not start syncing. Try again later.",
    }
}

/// A size in gigabytes with a whole number read plainly: "15 GB", not "15.0 GB".
fn storage_words(bytes: u64) -> String {
    size_words(bytes).replace(".0 ", " ")
}

/// "2.1 GB of 15 GB", or "2.1 GB used" when there is no limit.
fn quota_line(quota: &StorageQuota) -> String {
    match quota.total {
        Some(total) => format!("{} of {}", storage_words(quota.used), storage_words(total)),
        None => format!("{} used", storage_words(quota.used)),
    }
}

/// The line under the service: "Synced 5 minutes ago", "Syncing 3 items", "Paused".
fn sync_line(status: &SyncStatus) -> String {
    match (status.pause, status.anchor_age) {
        (SyncPause::Paused, _) => "Paused".to_owned(),
        _ if status.work() == SyncWork::Working => format!(
            "Syncing {} {}",
            status.pending,
            if status.pending == 1 { "item" } else { "items" }
        ),
        (SyncPause::Running, Some(age)) => format!("Synced {}", elapsed_words(age)),
        (SyncPause::Running, None) => "Waiting to sync".to_owned(),
    }
}

/// Both lines in one, as the row's second line: "Synced 5 minutes ago. 2.1 GB of 15 GB".
pub(crate) fn sync_lines(status: &SyncStatus) -> String {
    match &status.quota {
        Some(quota) => format!("{}. {}", sync_line(status), quota_line(quota)),
        None => sync_line(status),
    }
}

/// "$12.40".
fn cents_words(cents: Cents) -> String {
    format!("${}.{:02}", cents.0 / 100, cents.0 % 100)
}

/// "$12.40 of $50.00 this month".
pub(crate) fn spend_line(line: SpendLine) -> String {
    format!(
        "{} of {} this month",
        cents_words(line.spent),
        cents_words(line.cap)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::DatasetName;

    fn status(
        anchor_age: Option<i64>,
        pending: u64,
        pause: SyncPause,
        quota: Option<StorageQuota>,
    ) -> SyncStatus {
        SyncStatus {
            anchor_age,
            pending,
            conflicts: 0,
            pause,
            quota,
        }
    }

    #[test]
    fn a_dataset_says_when_it_synced_and_how_full_the_storage_is() {
        let some = |used, total| Some(StorageQuota { used, total });
        let cases = [
            (
                status(
                    Some(300),
                    0,
                    SyncPause::Running,
                    some(2_100_000_000, Some(15_000_000_000)),
                ),
                "Synced 5 minutes ago. 2.1 GB of 15 GB",
            ),
            (
                status(Some(10), 0, SyncPause::Running, some(500_000_000, None)),
                "Synced just now. 500 MB used",
            ),
            (
                status(Some(90_000), 0, SyncPause::Running, None),
                "Synced yesterday",
            ),
            (status(None, 0, SyncPause::Running, None), "Waiting to sync"),
            (
                status(Some(60), 3, SyncPause::Running, None),
                "Syncing 3 items",
            ),
            (
                status(Some(60), 1, SyncPause::Running, None),
                "Syncing 1 item",
            ),
            (status(Some(60), 3, SyncPause::Paused, None), "Paused"),
        ];
        for (held, want) in cases {
            assert_eq!(sync_lines(&held), want, "{held:?}");
        }
    }

    #[test]
    fn the_banner_counts_what_is_gone_or_says_all_of_it() {
        let hold = |discard, held| SyncHold {
            dataset: DatasetName("a/files".to_owned()),
            counts: HoldCounts { discard, held },
        };
        let cases = [
            (
                hold(30, 40),
                "30 of 40 files were removed on the server. Remove them here too?",
            ),
            (hold(40, 40), "All 40 files were removed on the server."),
        ];
        for (held, want) in cases {
            assert_eq!(hold_banner(&held), want, "{held:?}");
        }
    }

    #[test]
    fn each_way_sync_now_ends_has_its_own_plain_line() {
        let cases = [
            (SyncNowEnd::Started, "Syncing now."),
            (SyncNowEnd::Paused, "Syncing is paused for this account."),
            (
                SyncNowEnd::Failed,
                "Could not start syncing. Try again later.",
            ),
        ];
        for (end, want) in cases {
            assert_eq!(sync_now_line(end), want, "{end:?}");
        }
    }

    #[test]
    fn the_spend_line_reads_both_amounts() {
        let line = |spent, cap| SpendLine {
            spent: Cents(spent),
            cap: Cents(cap),
        };
        let cases = [
            (line(1240, 5000), "$12.40 of $50.00 this month"),
            (line(9000, 5000), "$90.00 of $50.00 this month"),
            (line(0, 5000), "$0.00 of $50.00 this month"),
        ];
        for (held, want) in cases {
            assert_eq!(spend_line(held), want, "{held:?}");
        }
    }
}
