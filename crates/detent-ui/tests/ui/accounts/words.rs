//! What the page says and how it looks: no developer words, the provider's mark, legible buttons.

use super::*;

#[test]
fn an_expiry_without_a_date_stays_a_read_only_line_in_the_modules_words() {
    let rows = vec![
        Held::new("work.example", "Work mail", "ok", &[])
            .expiring("Google signs this account out soon"),
    ];
    let mut w = world_of("accounts-expires", rows, Scheme::Light);
    open(&mut w, WORK);
    let row = "[aria-label=\"Sign-in\"]";
    assert_eq!(w.harness.count(row), 1, "{}", w.harness.html());
    let html = w.harness.html();
    assert!(html.contains("Google signs this account out soon"));
    assert!(html.contains("starts another 7"));
    for control in [".ds-toggle", ".ds-button"] {
        assert_eq!(
            w.harness.count(&format!("{row} {control}")),
            0,
            "the expiry row has no {control}"
        );
    }
    assert_eq!(
        w.harness.count("[data-accounts=\"attention\"]"),
        0,
        "no date, no banner"
    );
}

#[test]
fn the_pane_says_no_developer_words_with_the_modules_up_or_gone() {
    let sandbox = Sandbox::new("accounts-plain-no-bus");
    assert_no_developer_words(&sandbox.harness(accounts()));
    let mut harness = Sandbox::new("accounts-plain-none")
        .with_modules(FakeModules::new())
        .harness(accounts());
    settle(&mut harness, "the pane giving up", |h| {
        h.html().contains("The accounts service is not running")
    });
    assert_no_developer_words(&harness);
    let w = world("accounts-plain-up");
    assert_no_developer_words(&w.harness);
}

#[test]
fn the_account_type_is_drawn_with_the_providers_mark() {
    let fake = Accountd::new(
        Verdict::Allow,
        vec![
            Held::new("local-ai", "Ollama", "ok", &["llm"]).of_provider("ollama", "Ollama"),
            Held::new("mail", "Fastmail", "ok", &["mail"]).of_provider("fastmail", "Fastmail"),
        ],
    );
    let mut harness = Sandbox::new("accounts-mark")
        .with_modules(fake.hosted())
        .harness(accounts());
    settle(&mut harness, "the list", |h| h.count(".accounts-list") == 1);
    // The list draws each account's mark, and says the kind of account under its name.
    assert_eq!(
        harness.count(".accounts-list .ds-provider[data-kind=\"face\"]"),
        1
    );
    assert_eq!(
        harness.count(".accounts-list .ds-provider[data-kind=\"letter\"]"),
        1
    );
    assert!(
        harness
            .text_of(".accounts-list")
            .unwrap_or_default()
            .contains("Fastmail")
    );
    click(&mut harness, "#account-local-ai");
    settle(&mut harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some("local-ai")
    });
    assert_eq!(
        harness.count(".accounts-detail .ds-provider[data-kind=\"face\"]"),
        1,
        "a provider whose file gives a face is drawn from it:\n{}",
        harness.html()
    );
    click(&mut harness, "#account-mail");
    settle(&mut harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some("mail")
    });
    assert_eq!(
        harness.count(".accounts-detail .ds-provider[data-kind=\"letter\"]"),
        1,
        "one without a face keeps its named mark"
    );
}

#[test]
fn every_button_says_which_account_or_app_it_is_for() {
    let mut w = world("accounts-names");
    assert_eq!(
        w.harness.count(".ds-row [aria-label^=\"Manage\"]"),
        0,
        "a row is chosen by pressing it, with no button of its own"
    );
    detail(&mut w);
    for (selector, name) in [
        (SIGN_IN, "Sign in again Home cloud"),
        (REMOVE, "Remove account Home cloud"),
    ] {
        assert_eq!(
            w.harness.attr(selector, "aria-label").as_deref(),
            Some(name)
        );
    }
    open(&mut w, KEY);
    assert_eq!(
        w.harness.attr(REVOKE, "aria-label").as_deref(),
        Some("Revoke Photos allowed: llm")
    );
    assert_eq!(
        w.harness.count("[aria-label=\"Revoke\"]"),
        0,
        "no bare Revoke"
    );
}

#[test]
fn sign_in_again_says_a_browser_only_for_an_account_whose_row_says_browser() {
    let mut w = world("accounts-sign-in-words");
    detail(&mut w);
    let plain = w.harness.html();
    assert!(plain.contains("Sign in to the account again."), "{plain}");
    assert!(!plain.contains("in your browser"));
    open(&mut w, WORK);
    assert!(
        w.harness
            .html()
            .contains("Sign in to the account again in your browser.")
    );
}

#[test]
fn the_header_names_the_account_and_says_how_it_signs_in_only_when_the_module_does() {
    let mut w = world("accounts-header");
    // The account, and the words the module's `sign_in` row gives it (none: the row is missing).
    let cases = [(HOME, None), (WORK, Some("Signs in through the browser"))];
    for (account, how) in cases {
        open(&mut w, account);
        let words = w.harness.text_of(".accounts-header").unwrap_or_default();
        match how {
            Some(how) => assert!(words.contains(how), "{}: {words}", account.id),
            None => assert!(!words.contains("Signs in"), "{}: {words}", account.id),
        }
        let avatars = w.harness.count(".accounts-header .ds-avatar")
            + w.harness.count(".accounts-header .ds-provider");
        assert_eq!(avatars, 1, "{}: one large avatar", account.id);
    }
}
