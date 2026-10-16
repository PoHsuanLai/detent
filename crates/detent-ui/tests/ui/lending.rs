//! The switch "Let my other computers use this computer's models": it asks in plain words before
//! anything is written, Cancel leaves everything as it was, Allow sets sharing up and then writes
//! the setting, and turning it off takes sharing away. The service manager is a fake that writes
//! down its calls; the files are in a scratch folder.

use crate::support::Sandbox;
use crate::support::drive::{click, scroll_into_view};
use crate::support::lending::SHIPPED;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::{Driver, Harness, Query};
use std::time::Duration;

const LABEL: &str = "Let my other computers use this computer's models";
const ALLOW: &str = ".ds-alert .ds-button[data-answers=\"return\"]";
const CANCEL: &str = ".ds-alert .ds-button[data-answers=\"escape\"]";
const QUESTION: &str = "Let your other computers use this computer's models?";
const DROP_IN: &str = "systemd/user/inferd.service.d/tailnet.conf";

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

fn toggle() -> String {
    format!("[aria-label=\"{LABEL}\"] .ds-toggle")
}

/// The page's markup with an apostrophe read back, however the document spells it.
fn plain(harness: &Harness) -> String {
    harness.html().replace("&#39;", "'").replace("&#x27;", "'")
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fake daemons answer on real threads; a loaded machine running every test at once can
    // take several seconds, so the budget is generous (15 s) and costs nothing when it settles.
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

/// The window with the switch pressed once: the question is up.
fn asked(sandbox: &Sandbox) -> Harness {
    let mut harness = sandbox.harness(intelligence());
    scroll_into_view(&mut harness, &toggle());
    click(&mut harness, &toggle());
    settle(&mut harness, "the question", |h| {
        plain(h).contains(QUESTION)
    });
    harness
}

fn serve_setting(sandbox: &Sandbox) -> Option<String> {
    let text = sandbox.read("inferd/inferd.toml")?;
    let table: toml::Table = text.parse().expect("toml");
    table["ai"]["tailnet"]["serve"].as_str().map(str::to_owned)
}

fn checked(harness: &Harness) -> Option<String> {
    harness.attr(&toggle(), "aria-checked")
}

#[test]
fn the_question_is_asked_in_plain_words_before_anything_is_written() {
    let sandbox = Sandbox::new("lending-asks");
    let harness = asked(&sandbox);
    let html = plain(&harness);
    assert!(html.contains("your own Tailscale network"), "{html}");
    assert!(
        html.contains(">Allow<") && html.contains(">Cancel<"),
        "{html}"
    );
    assert_eq!(sandbox.drop_in(), None, "asking writes no file");
    assert_eq!(serve_setting(&sandbox), None, "asking sets nothing");
    assert!(
        sandbox.units().calls().is_empty(),
        "asking touches no service"
    );
    assert_eq!(checked(&harness).as_deref(), Some("false"));
}

#[test]
fn cancel_writes_nothing() {
    let sandbox = Sandbox::new("lending-cancel");
    let mut harness = asked(&sandbox);
    click(&mut harness, CANCEL);
    settle(&mut harness, "the question to go", |h| {
        !plain(h).contains(QUESTION)
    });
    assert_eq!(sandbox.drop_in(), None);
    assert_eq!(serve_setting(&sandbox), None);
    assert!(sandbox.units().calls().is_empty());
    assert_eq!(checked(&harness).as_deref(), Some("false"));
}

#[test]
fn allow_sets_sharing_up_and_then_writes_the_setting() {
    let sandbox = Sandbox::new("lending-allow");
    let mut harness = asked(&sandbox);
    click(&mut harness, ALLOW);
    settle(&mut harness, "the setting", |_| {
        serve_setting(&sandbox).is_some()
    });
    assert_eq!(serve_setting(&sandbox).as_deref(), Some("on"));
    assert_eq!(sandbox.drop_in().as_deref(), Some(SHIPPED));
    assert_eq!(
        sandbox.units().calls(),
        ["reload", "restart inferd.service"]
    );
    settle(&mut harness, "the switch on", |h| {
        checked(h).as_deref() == Some("true")
    });
}

#[test]
fn a_restart_that_fails_is_told_in_plain_words_and_the_setting_stays_off() {
    let sandbox = Sandbox::new("lending-refused");
    sandbox.units().refuse_restart();
    let mut harness = asked(&sandbox);
    click(&mut harness, ALLOW);
    settle(&mut harness, "the sentence", |h| {
        h.html().contains("the models service did not restart")
    });
    assert_eq!(serve_setting(&sandbox), None);
    assert_eq!(checked(&harness).as_deref(), Some("false"));
}

#[test]
fn a_different_file_is_named_in_the_question_and_replaced_on_allow() {
    let sandbox = Sandbox::new("lending-differs");
    sandbox.write(DROP_IN, "[Service]\nIPAddressAllow=10.0.0.0/8\n");
    let mut harness = asked(&sandbox);
    assert!(harness.html().contains("replaces it"), "{}", harness.html());
    click(&mut harness, ALLOW);
    settle(&mut harness, "the setting", |_| {
        serve_setting(&sandbox).is_some()
    });
    assert_eq!(sandbox.drop_in().as_deref(), Some(SHIPPED));
}

#[test]
fn turning_it_off_takes_sharing_away_without_asking() {
    let sandbox = Sandbox::new("lending-off");
    sandbox.write("inferd/inferd.toml", "[ai.tailnet]\nserve = \"on\"\n");
    sandbox.write(DROP_IN, SHIPPED);
    let mut harness = sandbox.harness(intelligence());
    scroll_into_view(&mut harness, &toggle());
    assert_eq!(checked(&harness).as_deref(), Some("true"));
    click(&mut harness, &toggle());
    settle(&mut harness, "the file to go", |_| {
        sandbox.drop_in().is_none()
    });
    settle(&mut harness, "the setting off", |_| {
        serve_setting(&sandbox).as_deref() == Some("off")
    });
    assert!(!plain(&harness).contains(QUESTION), "off asks nothing");
    assert_eq!(
        sandbox.units().calls(),
        ["reload", "restart inferd.service"]
    );
    settle(&mut harness, "the switch off", |h| {
        checked(h).as_deref() == Some("false")
    });
}
