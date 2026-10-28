//! Signing in again and removing an account, with the questions that come first.

use super::*;

#[test]
fn sign_in_again_runs_the_action_in_accountd_at_once() {
    let mut w = world("accounts-run");
    detail(&mut w);
    click(&mut w.harness, SIGN_IN);
    settle(&mut w.harness, "the action", |_| {
        !w.accountd.sets().is_empty()
    });
    assert_eq!(
        w.accountd.sets(),
        [(
            "accounts.home.reauth".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
}

#[test]
fn removing_an_account_asks_first_and_keeping_it_does_nothing() {
    let mut w = world("accounts-keep");
    let held = w.accountd.accounts.lock().expect("accounts").len();
    detail(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Home cloud?")
    });
    assert!(
        w.accountd.sets().is_empty(),
        "the press alone removes nothing"
    );
    click(
        &mut w.harness,
        ".ds-alert [data-role=\"normal\"], .ds-alert .ds-button",
    );
    settle(&mut w.harness, "the question to go", |h| {
        !h.html().contains("Remove account Home cloud?")
    });
    assert!(w.accountd.sets().is_empty());
    assert_eq!(w.accountd.accounts.lock().expect("accounts").len(), held);
}

#[test]
fn confirming_the_removal_removes_the_account_and_returns_to_the_list() {
    let mut w = world("accounts-remove");
    let held = w.accountd.accounts.lock().expect("accounts").len();
    detail(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Home cloud?")
    });
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-role=\"destructive\"]",
    );
    settle(&mut w.harness, "the removal", |_| {
        w.accountd.accounts.lock().expect("accounts").len() == held - 1
    });
    assert_eq!(
        w.accountd.sets(),
        [(
            "accounts.home.remove".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
    settle(&mut w.harness, "the list", |h| {
        h.count(".accounts-list") == 1 && !h.html().contains("Home cloud")
    });
    assert!(w.harness.html().contains("Work mail"));
    assert!(
        w.harness.html().contains("Home sync"),
        "syncd's account of the same id stays"
    );
}

#[test]
fn an_account_with_a_dotted_id_opens_and_removes_by_its_whole_id() {
    let mut w = world("accounts-dotted");
    click(&mut w.harness, WORK.row);
    settle(&mut w.harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some("work.example")
    });
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Work mail?")
    });
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-role=\"destructive\"]",
    );
    settle(&mut w.harness, "the set", |_| !w.accountd.sets().is_empty());
    assert_eq!(w.accountd.sets()[0].0, "accounts.work.example.remove");
}

#[test]
fn a_sign_in_that_ends_is_told_in_a_calm_line_and_the_rows_are_read_again() {
    let mut w = world("accounts-reauth-end");
    detail(&mut w);
    w.accountd.stop("home", "needs_reauth");
    w.announce(ModuleId::Accountd, "accounts.home.reauth", "failed");
    settle(&mut w.harness, "the line", |h| {
        h.html()
            .contains("Could not sign in to Home cloud. Try again later.")
    });
    settle(&mut w.harness, "the rows to be read again", |h| {
        h.html().contains("Needs sign-in")
    });
}

#[test]
fn the_remove_question_adds_a_clause_about_kept_files_only_when_a_sync_row_is_on() {
    let mut w = world("accounts-remove-files");
    open(&mut w, SYNC);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Home sync?")
    });
    assert!(!w.harness.html().contains("Files kept on this computer"));
    click(
        &mut w.harness,
        ".ds-alert .ds-button:not([data-role=\"destructive\"])",
    );
    settle(&mut w.harness, "the question to go", |h| {
        !h.html().contains("Remove account Home sync?")
    });
    click(&mut w.harness, KEEP_FILES);
    settle(&mut w.harness, "the switch on", |h| {
        h.attr(KEEP_FILES, "aria-checked").as_deref() == Some("true")
    });
    settle(&mut w.harness, "the pane to be free", |h| {
        h.attr(REMOVE, "data-availability").as_deref() != Some("disabled")
    });
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the question", |h| {
        h.html()
            .contains("Files kept on this computer for the account are deleted too.")
    });
}

#[test]
fn the_remove_answer_is_legible_in_light_and_dark() {
    for (scheme, tag) in [(Scheme::Light, "light"), (Scheme::Dark, "dark")] {
        let mut w = world_in(&format!("accounts-legible-{tag}"), scheme);
        detail(&mut w);
        click(&mut w.harness, REMOVE);
        settle(&mut w.harness, "the question", |h| {
            h.html().contains("Remove account Home cloud?")
        });
        let ratio = label_contrast(
            &mut w.harness,
            ".ds-alert .ds-button[data-role=\"destructive\"]",
        );
        assert!(
            ratio >= 4.5,
            "{tag}: Remove account is {ratio:.2}:1 on its fill"
        );
    }
}

#[test]
fn the_minus_waits_for_a_selection_then_asks_before_removing_the_selected_account() {
    let mut w = world("accounts-minus");
    assert_eq!(
        w.harness.attr(MINUS, "data-availability").as_deref(),
        Some("disabled"),
        "nothing is selected"
    );
    click(&mut w.harness, "[aria-posinset=\"3\"] .account-row-body");
    settle(&mut w.harness, "the minus", |h| {
        h.attr(MINUS, "data-availability").as_deref() != Some("disabled")
    });
    click(&mut w.harness, MINUS);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Remove account Work mail?")
    });
    assert!(
        w.accountd.sets().is_empty(),
        "the press alone removes nothing"
    );
}
