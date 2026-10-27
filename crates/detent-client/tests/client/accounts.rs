//! The Accounts flows: an account is added through the sheet, and "Sync now" tells the person how
//! it went and reads the datasets again.

use crate::support::builder;
use detent_client::{Heard, Notice};
use detent_model::{AccountsPane, AddEnd, DatasetName, SyncNowEnd};
use detent_system::accounts::{FakeAccounts, SheetAsked};
use detent_system::sync::{FakeSync, SyncAsked};
use std::sync::Arc;

#[tokio::test]
async fn adding_an_account_opens_the_sheet_and_answers_how_it_ended() {
    let link = FakeAccounts::answering(AccountsPane::default());
    link.sheets_end(AddEnd::Added);
    let client = builder().with_accounts(Arc::new(link.clone())).build();

    assert_eq!(client.add_account().await, AddEnd::Added);
    assert_eq!(client.accounts().add_account().await, AddEnd::Added);
    assert_eq!(link.asked(), [SheetAsked::Account, SheetAsked::Account]);
}

#[tokio::test]
async fn sync_now_is_asked_of_the_service_and_the_person_is_told() {
    let sync = FakeSync::knowing(Vec::new());
    let client = builder().with_sync(Arc::new(sync.clone())).build();
    let datasets = client.accounts().datasets().clone();
    let dataset = DatasetName("account-1/mail".to_owned());

    datasets.sync_now(dataset.clone()).await;

    assert_eq!(sync.asked(), [SyncAsked::SyncNow(dataset)]);
    let events = datasets.live().events();
    let mut told = None;
    while let Some(heard) = events.next().await {
        if let Heard::Notice(notice) = heard {
            told = Some(notice);
            break;
        }
    }
    assert_eq!(told, Some(Notice::SyncNow(SyncNowEnd::Started)));
}
