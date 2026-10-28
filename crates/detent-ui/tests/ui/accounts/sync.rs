//! Sync rows: the Show in Files switch, the sync switches, Continue as, and a dataset's standing.

use super::*;

const SHOW_IN_FILES: &str = "[aria-label=\"Show in Files\"] .ds-toggle";
const FILES_NOT_ON: &str = "Files wasn't turned on. You can try again.";

#[test]
fn show_in_files_is_syncds_switch_with_its_line_under_it_and_a_turn_on_is_a_set() {
    let mirrored = vec![
        Held::new("home", "Home sync", "ok", &[])
            .syncing()
            .showing_in_files(FILES_NOT_ON),
    ];
    let mut w = start_syncing(
        "accounts-show-in-files",
        Scheme::Light,
        identities(),
        &[ModuleId::Accountd, ModuleId::Syncd],
        mirrored,
        Syncd::default(),
    );
    open(&mut w, SYNC);
    assert_eq!(
        w.harness.attr(SHOW_IN_FILES, "aria-checked").as_deref(),
        Some("false")
    );
    // The service's line stands under the switch, not behind the info button.
    assert!(
        w.harness
            .text_of("[aria-label=\"Show in Files\"]")
            .unwrap_or_default()
            .contains(FILES_NOT_ON),
        "{}",
        w.harness.html()
    );
    click(&mut w.harness, SHOW_IN_FILES);
    settle(&mut w.harness, "the set", |_| !w.syncd.sets().is_empty());
    assert_eq!(
        w.syncd.sets(),
        [(
            "accounts.home.storage.files".to_owned(),
            toml::Value::String("on".to_owned())
        )]
    );
}

#[test]
fn the_sync_switches_are_syncds_and_set_on_and_off_in_syncds_module() {
    let mut w = world("accounts-sync");
    open(&mut w, SYNC);
    let html = w.harness.html();
    assert!(html.contains("Upload new photos from a folder on this computer to Google Photos"));
    assert!(
        html.contains("never reads or deletes anything else"),
        "the sync rows keep their help text"
    );
    assert_eq!(
        w.harness.attr(KEEP_FILES, "aria-checked").as_deref(),
        Some("false")
    );
    click(&mut w.harness, KEEP_FILES);
    settle(&mut w.harness, "the switch on", |h| {
        h.attr(KEEP_FILES, "aria-checked").as_deref() == Some("true")
    });
    settle(&mut w.harness, "the pane to be free", |h| {
        h.attr(UPLOAD_PHOTOS, "data-availability").as_deref() != Some("disabled")
    });
    click(&mut w.harness, KEEP_FILES);
    settle(&mut w.harness, "the second set", |_| {
        w.syncd.sets().len() == 2
    });
    let want = |word: &str| {
        (
            "accounts.home.sync.files".to_owned(),
            toml::Value::String(word.to_owned()),
        )
    };
    assert_eq!(w.syncd.sets(), [want("on"), want("off")]);
    assert!(
        w.accountd.sets().is_empty(),
        "accountd, which has an account of the same id, heard nothing"
    );
}

#[test]
fn continue_as_runs_syncds_row_on_one_press() {
    let mut w = world("accounts-continue-as");
    click(&mut w.harness, CONTINUE_AS);
    settle(&mut w.harness, "the action", |_| !w.syncd.sets().is_empty());
    assert_eq!(
        w.syncd.sets(),
        [("accounts.sync.add".to_owned(), toml::Value::Boolean(true))]
    );
    assert!(w.accountd.sets().is_empty() && w.inferd.sets().is_empty());
}

/// syncd's account with its files switch on, so the sync service keeps `home/files`.
fn mirrored_files() -> Vec<Held> {
    let mut held = Held::new("home", "Home sync", "ok", &[]).syncing();
    held.sync[0].1 = true;
    vec![held]
}

const FILES: &str = "home/files";
const RING: &str = ".ds-row .ds-progress[data-style=\"ring\"]";

#[test]
fn a_synced_dataset_says_when_it_synced_and_how_full_the_storage_is_and_a_ring_turns_while_it_works()
 {
    let idle = Standing {
        anchor_age: 300,
        pending: 0,
        conflicts: 0,
        used: 2_100_000_000,
        total: Some(15_000_000_000),
    };
    let sync1 = Syncd::default();
    sync1.stands(FILES, idle);
    let mut w = start_syncing(
        "accounts-sync-status",
        Scheme::Light,
        identities(),
        &[ModuleId::Accountd, ModuleId::Syncd],
        mirrored_files(),
        sync1,
    );
    open(&mut w, SYNC);
    settle(&mut w.harness, "the sync line", |h| {
        h.html().contains("Synced 5 minutes ago. 2.1 GB of 15 GB")
    });
    assert_eq!(w.harness.count(RING), 0, "nothing is moving");

    // Items are left to send: the avatar in the list and the one at the head of the detail turn.
    w.sync1.stands(FILES, Standing { pending: 4, ..idle });
    w.announce(ModuleId::Syncd, "accounts.home.sync.files", "on");
    settle(&mut w.harness, "the ring", |h| h.count(RING) == 1);
    assert_eq!(
        w.harness
            .count("#account-sync-home .ds-progress[data-style=\"ring\"]"),
        1,
        "the ring is on the account that syncs"
    );
    assert_eq!(
        w.harness
            .count(".accounts-header .ds-progress[data-style=\"ring\"]"),
        1
    );
    assert!(w.harness.html().contains("Syncing 4 items"));

    // It catches up: the ring goes away and the line is the age again.
    w.sync1.stands(FILES, idle);
    w.announce(ModuleId::Syncd, "accounts.home.sync.files", "on");
    settle(&mut w.harness, "the ring to stop", |h| h.count(RING) == 0);
}
