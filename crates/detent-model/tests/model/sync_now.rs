//! "Sync now": the dataset a sync row names, and the words for how the service took it.

use detent_model::{AccountKey, DatasetName, SyncNowEnd};
use ds_settings::schema::KeyPath;

#[test]
fn a_sync_row_names_its_dataset_by_the_accounts_bus_segment() {
    let row = KeyPath("accounts.work.example.sync.photos".to_owned());
    assert_eq!(
        DatasetName::of_switch(&AccountKey("work.example".to_owned()), &row),
        DatasetName("work_example/photos".to_owned())
    );
}

#[test]
fn each_end_has_its_own_plain_line_and_a_pause_is_not_a_failure() {
    let lines = [SyncNowEnd::Started, SyncNowEnd::Paused, SyncNowEnd::Failed].map(SyncNowEnd::line);
    assert_eq!(lines[1], "Syncing is paused for this account.");
    assert!(lines[0] != lines[1] && lines[1] != lines[2] && lines[0] != lines[2]);
}
