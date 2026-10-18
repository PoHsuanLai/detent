//! A request from outside to show an account (D-Bus activation's `open-account`): the window
//! moves to that account's detail, or to the list when the account is not known.

use crate::support::Sandbox;
use crate::support::accountd::{Accountd, Held, Policy};
use crate::support::bus::PrivateBus;
use crate::support::drive::page_of;
use detent_model::{AccountSegment, AccountTarget, Notice, PageSlug};
use detent_ui::Opening;
use ds_harness::Harness;
use std::time::Duration;

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

fn detail(segment: &str) -> Notice {
    Notice::OpenAccount(AccountTarget::Detail(AccountSegment(segment.to_owned())))
}

/// The window on another page, over a fake accountd holding two accounts.
fn world(name: &str) -> (Harness, detent_model::Outbox, impl Sized) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fake daemon");
    let bus = PrivateBus::start(name);
    let fake = Accountd::new(
        Policy::Allow,
        vec![
            Held::new("home", "Home cloud", "ok", &["storage"]),
            Held::new("work.example", "Work mail", "ok", &["mail"]),
        ],
    );
    let (daemon, served) = rt.block_on(fake.serve_announcing(&bus));
    let opening = Opening {
        page: Some(PageSlug("dock".to_owned())),
        query: String::new(),
    };
    let (harness, outbox) = Sandbox::new(name)
        .on_bus(&bus.address)
        .harness_told(opening);
    (harness, outbox, (rt, bus, daemon, served, fake))
}

#[test]
fn an_account_segment_opens_that_accounts_detail() {
    let (mut harness, outbox, _keep) = world("notice-account");
    outbox.send(detail("work_example")).expect("listening");
    settle(&mut harness, "the account", |h| {
        h.attr(".accounts-detail", "data-account").as_deref() == Some("work.example")
    });
    assert_eq!(page_of(&harness).as_deref(), Some("accounts"));
}

#[test]
fn an_unknown_account_opens_the_list() {
    let (mut harness, outbox, _keep) = world("notice-unknown-account");
    outbox.send(detail("nobody")).expect("listening");
    settle(&mut harness, "the list", |h| h.count(".accounts-list") == 1);
    assert_eq!(page_of(&harness).as_deref(), Some("accounts"));
}

#[test]
fn the_list_request_leaves_an_open_account() {
    let (mut harness, outbox, _keep) = world("notice-account-list");
    outbox.send(detail("home")).expect("listening");
    settle(&mut harness, "the account", |h| {
        h.count(".accounts-detail") == 1
    });
    outbox
        .send(Notice::OpenAccount(AccountTarget::List))
        .expect("listening");
    settle(&mut harness, "the list", |h| h.count(".accounts-list") == 1);
}

#[test]
fn raising_is_taken_without_a_window_handle_to_ask() {
    let (mut harness, outbox, _keep) = world("notice-raise");
    outbox
        .send(Notice::Raise(Some("token".to_owned())))
        .expect("listening");
    harness.advance(Duration::from_millis(400));
    assert_eq!(page_of(&harness).as_deref(), Some("dock"));
}
