//! The held deletions and their one transition.

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
fn nothing_held_refreshes_and_a_refusal_is_told() {
    let dataset = files("home");
    let (_, effects) = step(Holds::default(), HoldInput::NothingHeld(dataset.clone()));
    assert_eq!(effects, [HoldEffect::Refresh(dataset.clone())]);
    let (_, effects) = step(Holds::default(), HoldInput::Refused(dataset));
    assert_eq!(effects, [HoldEffect::CouldNotRemove]);
}

#[test]
fn a_hold_belongs_to_its_account_and_sits_on_its_sync_row() {
    let sync = |leaf: &str| AccountSync {
        path: ds_settings::schema::KeyPath(format!("accounts.work.example.sync.{leaf}")),
        label: leaf.to_owned(),
        help: String::new(),
        on: SwitchState::On,
    };
    let account = |sync: Vec<AccountSync>| {
        let mut pane = AccountsPane::default();
        pane.accounts.push(Account {
            key: AccountKey("work.example".to_owned()),
            module: crate::accounts::ModuleId::Accountd,
            label: "Work".to_owned(),
            status: AccountStatus::Ok,
            place: Place::Elsewhere,
            expires: None,
            sign_in: None,
            provider: None,
            group: porter_core::sheet::ProviderGroup::Internet,
            services: Vec::new(),
            sync,
            show_in_files: None,
            permissions: Vec::new(),
            reauth: None,
            sign_out: None,
            remove: None,
            used_by: Vec::new(),
            spend: AccountSpend::default(),
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

#[test]
fn a_service_that_tells_too_many_holds_is_not_believed() {
    use super::hold_step::HOLDS_MAX;
    let mut holds = Holds::default();
    for number in 0..HOLDS_MAX + 20 {
        let (next, _) = step(holds, told(&files(&format!("account{number}")), 1, 2));
        holds = next;
    }
    assert_eq!(holds.0.len(), HOLDS_MAX, "the banners stop at the bound");
    // A hold already kept still follows the service: updated, then ended.
    let kept = files("account0");
    let (holds, _) = step(holds, told(&kept, 2, 2));
    assert_eq!(holds.0.len(), HOLDS_MAX);
    assert!(
        holds
            .0
            .iter()
            .any(|hold| hold.dataset == kept && hold.counts.discard == 2)
    );
    let (holds, _) = step(
        holds,
        HoldInput::Told {
            dataset: kept,
            counts: None,
        },
    );
    assert_eq!(holds.0.len(), HOLDS_MAX - 1);
}

#[test]
fn a_dataset_name_over_its_bound_is_not_taken() {
    use super::hold::DATASET_NAME_MAX;
    let longest = "a".repeat(DATASET_NAME_MAX);
    assert_eq!(
        DatasetName::bounded(&longest),
        Some(DatasetName(longest.clone()))
    );
    assert_eq!(DatasetName::bounded(&format!("{longest}a")), None);
}
