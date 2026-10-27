//! AI keys: permits, switches, and the month's spend.

use super::*;

#[test]
fn an_ai_key_shows_inferds_permits_with_revoke_and_revoking_goes_to_inferd() {
    let mut w = world("accounts-permit");
    open(&mut w, KEY);
    let html = w.harness.html();
    assert!(html.contains("Photos allowed: llm"), "{html}");
    assert_eq!(
        w.harness.attr(REVOKE, "aria-label").as_deref(),
        Some("Revoke Photos allowed: llm")
    );
    press_revoke(&mut w);
    settle(&mut w.harness, "the revoke", |_| {
        !w.inferd.sets().is_empty()
    });
    assert_eq!(
        w.inferd.sets(),
        [(
            "accounts.claude-key.grant.g1".to_owned(),
            toml::Value::Boolean(true)
        )]
    );
    assert!(w.accountd.sets().is_empty() && w.syncd.sets().is_empty());
    settle(&mut w.harness, "the permit to leave the page", |h| {
        !h.html().contains("Photos allowed: llm")
    });
}

#[test]
fn a_switch_sets_the_service_in_inferd_and_the_answer_stays_on_screen() {
    let mut w = world("accounts-switch");
    open(&mut w, KEY);
    assert_eq!(w.harness.attr(LLM, "aria-checked").as_deref(), Some("true"));
    click(&mut w.harness, LLM);
    settle(&mut w.harness, "the set", |_| !w.inferd.sets().is_empty());
    assert_eq!(
        w.inferd.sets(),
        [(
            "accounts.claude-key.service.llm".to_owned(),
            toml::Value::String("off".to_owned())
        )]
    );
    settle(&mut w.harness, "the switch to read off", |h| {
        h.attr(LLM, "aria-checked").as_deref() == Some("false")
    });
}

#[test]
fn a_session_permit_row_comes_and_goes_with_inferds_announcement() {
    let mut w = world("accounts-session-grant");
    open(&mut w, KEY);
    let sentence = "Claude Code can use Llm (this session)";
    let (grant, path) = ("s1", "accounts.claude-key.grant.s1");
    w.inferd
        .accounts
        .lock()
        .expect("accounts")
        .iter_mut()
        .for_each(|held| held.grants.push((grant.to_owned(), sentence.to_owned())));
    w.announce(Module::Inferd, path, "added");
    settle(&mut w.harness, "the session row", |h| {
        h.html().contains(sentence)
    });
    w.inferd
        .accounts
        .lock()
        .expect("accounts")
        .iter_mut()
        .for_each(|held| held.grants.retain(|(g, _)| g != grant));
    w.announce(Module::Inferd, path, "removed");
    settle(&mut w.harness, "the session row to go", |h| {
        !h.html().contains(sentence)
    });
}

#[test]
fn a_permit_says_the_day_it_was_allowed_or_that_it_lasts_a_session_from_the_rows_keyed_by_its_grant()
 {
    let mut w = world("accounts-grant-facts");
    {
        let mut held = w.inferd.accounts.lock().expect("accounts");
        let key = held.first_mut().expect("the key");
        *key = key
            .clone()
            .granted_on("g1", "2026-10-08", "always")
            .granting("g2", "Mail allowed: llm")
            .granted_on("g2", "2026-10-09", "session");
    }
    w.announce(Module::Inferd, "accounts.claude-key.grant.g2", "added");
    open(&mut w, KEY);
    settle(&mut w.harness, "the second permit", |h| {
        h.html().contains("This session")
    });
    assert!(w.harness.html().contains("Allowed on 8 Oct"));
}

#[test]
fn an_ai_account_shows_this_months_spend_against_its_cap_as_a_bar() {
    let mut w = world("accounts-spend");
    open(&mut w, KEY);
    let bar = "[aria-label=\"Spent this month\"]";
    assert_eq!(
        w.harness.attr(bar, "aria-valuenow").as_deref(),
        Some("50"),
        "{}",
        w.harness.html()
    );
    assert!(w.harness.html().contains("$25.00 of $50.00 this month"));
    // An account that publishes no spend rows has no bar.
    open(&mut w, HOME);
    assert_eq!(w.harness.count(bar), 0);
}
