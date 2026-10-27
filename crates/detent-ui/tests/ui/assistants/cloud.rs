//! Where the assistant may run: this computer, the person's computers and cloud accounts.

use super::*;

fn the_cloud_account_is_listed(w: &mut World) {
    settle(&mut w.harness, "the account", |h| h.count(CLOUD) == 1);
}

#[test]
fn this_computer_is_always_on_with_no_switch_and_a_cloud_account_starts_off() {
    let mut w = world("assistants-groups", vec![acme()], None);
    the_cloud_account_is_listed(&mut w);
    let html = w.harness.html();
    for text in [
        "On this computer",
        "Your computers",
        "Cloud accounts",
        "Always on",
        "Acme",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(
        w.harness.count("[aria-label=\"This computer\"] .ds-toggle"),
        0
    );
    assert_eq!(
        w.harness.attr(CLOUD, "aria-checked").as_deref(),
        Some("false")
    );
    assert_eq!(written(&w.sandbox), None, "reading writes nothing");
}

#[test]
fn turning_a_cloud_account_on_asks_in_plain_words_and_yes_writes_it() {
    let mut w = world("assistants-confirm", vec![acme()], None);
    the_cloud_account_is_listed(&mut w);
    click(&mut w.harness, CLOUD);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Turn on me@acme.example?")
    });
    let html = w.harness.html().replace("&#39;", "'");
    assert!(
        html.contains("Requests the assistant can't do on your computers may be sent to Acme."),
        "{html}"
    );
    assert_eq!(written(&w.sandbox), None, "the press alone writes nothing");
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-answers=\"return\"]",
    );
    settle(&mut w.harness, "the write", |_| {
        written(&w.sandbox).is_some()
    });
    let places = written(&w.sandbox).expect("written");
    assert_eq!(places.cloud.len(), 1);
    assert_eq!(places.cloud[0].allowed, CloudAllowed::On);
}

#[test]
fn cancel_leaves_the_account_off_and_writes_nothing() {
    let mut w = world("assistants-cancel", vec![acme()], None);
    the_cloud_account_is_listed(&mut w);
    click(&mut w.harness, CLOUD);
    settle(&mut w.harness, "the question", |h| {
        h.html().contains("Turn on me@acme.example?")
    });
    click(
        &mut w.harness,
        ".ds-alert .ds-button[data-answers=\"escape\"]",
    );
    settle(&mut w.harness, "the question to go", |h| {
        !h.html().contains("Turn on me@acme.example?")
    });
    assert_eq!(written(&w.sandbox), None);
    assert_eq!(
        w.harness.attr(CLOUD, "aria-checked").as_deref(),
        Some("false")
    );
}

#[test]
fn a_computer_of_the_persons_switches_without_a_question_and_the_file_follows() {
    let file = "[assistant]\nthis_computer = {}\n\n[[assistant.own_computers]]\nname = \"lab-server\"\nallowed = \"on\"\n";
    let mut c = computers_in(
        "assistants-own",
        Some(file),
        vec![LAB_SERVER],
        vec![],
        vec![],
    );
    settle(&mut c.harness, "the computer", |h| h.count(LAB) == 1);
    assert_eq!(c.harness.attr(LAB, "aria-checked").as_deref(), Some("true"));
    click(&mut c.harness, LAB);
    settle(&mut c.harness, "the write", |_| {
        written(&c.sandbox).is_some_and(|places| places.own[0].allowed == Toggle::Off)
    });
    assert!(!c.harness.html().contains("Turn on lab-server?"));
}

#[test]
fn ask_me_each_time_sits_under_a_cloud_rows_advanced_and_writes_ask() {
    let file = "[assistant]\nthis_computer = {}\n\n[[assistant.cloud_accounts]]\nname = \"me@acme.example\"\nallowed = \"on\"\nprovider = \"Acme\"\n";
    let mut w = world("assistants-ask", vec![acme()], Some(file));
    the_cloud_account_is_listed(&mut w);
    assert_eq!(
        w.harness.attr(CLOUD, "aria-checked").as_deref(),
        Some("true")
    );
    click(
        &mut w.harness,
        ".ds-disclosure-trigger[aria-label=\"Advanced, me@acme.example\"]",
    );
    let ask = "[aria-label=\"Ask me each time\"] .ds-toggle";
    settle(&mut w.harness, "the switch", |h| h.count(ask) == 1);
    assert_eq!(
        w.harness.count(
            "[aria-label=\"Advanced, This computer\"] ~ * [aria-label=\"Ask me each time\"]"
        ),
        0,
        "this computer has no such switch"
    );
    click_when_still(&mut w.harness, ask);
    settle(&mut w.harness, "the write", |_| {
        written(&w.sandbox)
            .is_some_and(|places| places.cloud[0].allowed == CloudAllowed::AskEachTime)
    });
}

#[test]
fn an_assistant_that_signs_itself_in_keeps_its_own_row_apart_from_the_places() {
    let claude = Held::new("claude", "Claude Code", "needs_login", &["agent"])
        .in_group("agent")
        .signing_in("agent");
    let mut w = world("assistants-external", vec![acme(), claude], None);
    settle(&mut w.harness, "the assistants", |h| {
        h.count("[data-assistants=\"coding\"]") == 1
    });
    let html = w.harness.html();
    let others = &html[html.find("data-assistants=\"coding\"").expect("the group")..];
    assert!(others.contains("Claude Code") && others.contains("Needs sign-in"));
    assert!(!others.contains("me@acme.example"), "no combined list");
    click(&mut w.harness, "[aria-label=\"Sign in Claude Code\"]");
    settle(&mut w.harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
}

#[test]
fn an_assistant_docket_and_accountd_both_know_is_one_row_with_both_facts() {
    let claude = Held::new("claude", "Claude Code", "needs_login", &["agent"])
        .in_group("agent")
        .signing_in("agent");
    let mut w = world_with("assistants-joined", vec![claude], None, Some(AGENTS));
    let group = "[data-assistants=\"coding\"]";
    // The row is there from docket's list; accountd's status joins it once its list arrives.
    settle(&mut w.harness, "the status", |h| {
        h.text_of(group)
            .is_some_and(|t| t.contains("Needs sign-in"))
    });
    let text = w.harness.text_of(group).unwrap_or_default();
    assert_eq!(text.matches("Claude Code").count(), 1, "one row: {text}");
    for want in ["Needs sign-in", "The agent\u{2019}s own choice"] {
        assert!(text.contains(want), "{want} in {text}");
    }
}
