//! The held deletions: the banner's words and the one transition.

use super::*;

fn files(account: &str) -> DatasetName {
    DatasetName(format!("{account}/files"))
}

fn told(dataset: &DatasetName, discard: u64, held: u64) -> HoldInput {
    HoldInput::Told {
        dataset: dataset.clone(),
        counts: Some(HoldCounts { discard, held }),
    }
}

fn step(holds: Holds, input: HoldInput) -> (Holds, Vec<HoldEffect>) {
    hold_step(holds, input)
}

#[test]
fn the_banner_counts_what_is_gone_or_says_all_of_it() {
    let some = SyncHold {
        dataset: files("a"),
        counts: HoldCounts {
            discard: 30,
            held: 40,
        },
    };
    assert_eq!(
        some.banner(),
        "30 of 40 files were removed on the server. Remove them here too?"
    );
    let all = SyncHold {
        counts: HoldCounts {
            discard: 40,
            held: 40,
        },
        ..some
    };
    assert_eq!(all.banner(), "All 40 files were removed on the server.");
}

#[test]
fn a_hold_comes_with_its_signal_and_goes_with_the_empty_one() {
    let dataset = files("home");
    let (holds, effects) = step(Holds::default(), told(&dataset, 3, 4));
    assert!(effects.is_empty());
    assert_eq!(holds.0.len(), 1);
    // Told again with other counts: replaced, not doubled.
    let (holds, _) = step(holds, told(&dataset, 4, 4));
    assert_eq!(holds.0.len(), 1);
    assert_eq!(holds.0[0].counts.discard, 4);
    let (holds, effects) = step(
        holds,
        HoldInput::Told {
            dataset,
            counts: None,
        },
    );
    assert!(holds.0.is_empty());
    assert!(effects.is_empty());
}

#[test]
fn keep_calls_nothing_and_remove_asks_the_service_to_discard() {
    let dataset = files("home");
    let (holds, _) = step(Holds::default(), told(&dataset, 3, 4));
    let (kept, effects) = step(holds.clone(), HoldInput::Keep(dataset.clone()));
    assert!(effects.is_empty(), "Keep calls nothing");
    assert!(kept.0.is_empty(), "the banner gets out of the way");
    let (still, effects) = step(holds, HoldInput::Remove(dataset.clone()));
    assert_eq!(effects, [HoldEffect::Confirm(dataset.clone())]);
    assert_eq!(
        still.0.len(),
        1,
        "the banner stays until the service says the hold ended"
    );
    // Remove for a dataset nothing is held on asks nothing.
    let (_, effects) = step(Holds::default(), HoldInput::Remove(dataset));
    assert!(effects.is_empty());
}

#[test]
fn nothing_held_refreshes_and_a_refusal_is_told_calmly() {
    let dataset = files("home");
    let (_, effects) = step(Holds::default(), HoldInput::NothingHeld(dataset.clone()));
    assert_eq!(effects, [HoldEffect::Refresh(dataset.clone())]);
    let (_, effects) = step(Holds::default(), HoldInput::Refused(dataset));
    assert_eq!(
        effects,
        [HoldEffect::Tell(
            "Could not remove the files. Try again later.".to_owned()
        )]
    );
}

#[test]
fn a_hold_belongs_to_its_account_and_sits_on_its_sync_row() {
    let sync = |leaf: &str| AccountSync {
        path: ds_settings::schema::KeyPath(format!("accounts.work.example.sync.{leaf}")),
        label: leaf.to_owned(),
        help: String::new(),
        on: true,
    };
    let account = |sync: Vec<AccountSync>| {
        let mut pane = AccountsPane::default();
        pane.accounts.push(Account {
            key: AccountKey("work.example".to_owned()),
            label: "Work".to_owned(),
            status: AccountStatus::Ok,
            place: Place::Elsewhere,
            expires: None,
            sign_in: None,
            provider: None,
            services: Vec::new(),
            sync,
            permissions: Vec::new(),
            reauth: None,
            sign_out: None,
            remove: None,
        });
        pane.accounts.remove(0)
    };
    let both = account(vec![sync("files"), sync("photos")]);
    let holds = Holds(vec![
        SyncHold {
            dataset: DatasetName("work_example/files".to_owned()),
            counts: HoldCounts {
                discard: 1,
                held: 2,
            },
        },
        SyncHold {
            dataset: DatasetName("work_example/photos_originals".to_owned()),
            counts: HoldCounts {
                discard: 2,
                held: 2,
            },
        },
        SyncHold {
            dataset: DatasetName("other/files".to_owned()),
            counts: HoldCounts {
                discard: 1,
                held: 1,
            },
        },
    ]);
    assert!(holds.any_of(&both));
    let on = |row: &AccountSync| holds.on_switch(&both, &row.path).count();
    assert_eq!(on(&both.sync[0]), 1, "files");
    assert_eq!(
        on(&both.sync[1]),
        1,
        "photos_originals sits on the photos row"
    );
    assert_eq!(holds.on_no_switch(&both).count(), 0);
    let files_only = account(vec![sync("files")]);
    assert_eq!(
        holds.on_no_switch(&files_only).count(),
        1,
        "a dataset no row controls is shown after the rows"
    );
}
