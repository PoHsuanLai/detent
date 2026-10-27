//! Adding an account, and the sign-in step of the setup window.

use super::*;

#[test]
fn the_list_has_an_add_account_button_and_a_refused_ask_is_told_calmly() {
    let mut w = world("accounts-add");
    assert_eq!(w.harness.count(ADD), 1);
    // The tip is the short name; the spoken name keeps the ellipsis.
    assert_eq!(
        w.harness.attr(ADD, "aria-description").as_deref(),
        Some("Add Account")
    );
    assert_eq!(
        w.harness.attr(MINUS, "data-tip").as_deref(),
        Some("Remove Account")
    );
    click(&mut w.harness, ADD);
    settle(&mut w.harness, "the line", |h| {
        h.html()
            .contains("Could not add an account. Try again later.")
    });
    // The button is free again after the sheet ended.
    settle(&mut w.harness, "the button", |h| {
        h.attr(ADD, "data-availability").as_deref() != Some("disabled")
    });
}

fn setup_opening() -> Opening {
    Opening {
        page: Some(PageSlug("accounts-setup".to_owned())),
        query: String::new(),
    }
}

#[test]
fn the_sign_in_step_lists_what_is_signed_in_and_offers_add_account_and_skip() {
    const ADD_ACCOUNT: &str = ".accounts-setup > [aria-label=\"Add Account\u{2026}\"]";
    const SKIP: &str = ".accounts-skip";
    const CONTINUE: &str = ".accounts-continue";
    // (the world, the accounts held, whether Continue waits)
    let cases = [
        ("accounts-setup-nothing", Vec::new(), true),
        ("accounts-setup-signed-in", identities(), false),
    ];
    for (name, rows, waits) in cases {
        let fake = Accountd::new(Policy::Allow, rows.clone());
        let mut harness = Sandbox::new(name)
            .with_modules(fake.hosted())
            .harness(setup_opening());
        settle(&mut harness, "the step's rows", |h| {
            h.count(".accounts-setup .account-row-body") == rows.len()
                && h.count(".accounts-setup") == 1
        });
        let disabled = |h: &Harness, button: &str| {
            h.attr(button, "data-availability").as_deref() == Some("disabled")
        };
        assert!(!disabled(&harness, SKIP), "{name}: Skip is always there");
        assert_eq!(disabled(&harness, CONTINUE), waits, "{name}");
        assert_eq!(
            harness.html().contains("Nothing is signed in yet"),
            rows.is_empty(),
            "{name}"
        );
        for held in &rows {
            assert!(
                harness.html().contains(&held.label),
                "{name}: {}",
                held.label
            );
        }
        // Add Account… is the same ask the page makes; a service that refuses is told calmly.
        click(&mut harness, ADD_ACCOUNT);
        settle(&mut harness, "the line", |h| {
            h.html()
                .contains("Could not add an account. Try again later.")
        });
        settle(&mut harness, "the button", |h| {
            h.attr(ADD_ACCOUNT, "data-availability").as_deref() != Some("disabled")
        });
        // Skip leaves nothing behind.
        click(&mut harness, SKIP);
        assert!(fake.sets().is_empty(), "{name}");
        let text = harness.text_of(".accounts-setup").unwrap_or_default();
        for word in ["accountd", "OAuth", "daemon", "MCP", "ACP"] {
            assert!(!text.contains(word), "{name}: {word} is in: {text}");
        }
    }
}
