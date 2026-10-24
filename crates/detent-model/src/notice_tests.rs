//! The window's inbox cannot be made to grow by whoever calls its activation object.

use crate::notice::{NOTICE_INBOX, Notice, inbox};

#[test]
fn a_flood_of_notices_leaves_exactly_the_bounded_queue() {
    let (outbox, inbox) = inbox();
    for _ in 0..10_000 {
        assert!(
            outbox.send(Notice::Reload).is_ok(),
            "a full inbox drops, it does not fail"
        );
    }
    assert_eq!(outbox.dropped(), 10_000 - NOTICE_INBOX);
    let mut waiting = inbox.take().expect("the receiver");
    let mut count = 0;
    while waiting.try_recv().is_some() {
        count += 1;
    }
    assert_eq!(count, NOTICE_INBOX, "the queue holds the first notices");
}

#[test]
fn the_count_of_dropped_notices_starts_again_once_the_window_caught_up() {
    let (outbox, inbox) = inbox();
    for _ in 0..NOTICE_INBOX + 3 {
        outbox.send(Notice::Reload).expect("sent");
    }
    assert_eq!(outbox.dropped(), 3);
    let mut waiting = inbox.take().expect("the receiver");
    while waiting.try_recv().is_some() {}
    outbox.send(Notice::Reload).expect("sent");
    assert_eq!(outbox.dropped(), 0, "a flood that ended is forgotten");
}

#[test]
fn a_window_that_has_gone_is_told_so() {
    let (outbox, inbox) = inbox();
    drop(inbox.take());
    assert!(outbox.send(Notice::Reload).is_err());
}

#[test]
fn a_refilled_inbox_gives_the_next_window_its_own_queue() {
    let (first, inbox) = inbox();
    drop(inbox.take());
    assert!(
        first.send(Notice::Reload).is_err(),
        "the first window has gone"
    );
    let second = inbox.refill();
    second
        .send(Notice::Reload)
        .expect("the next window is waiting");
    let mut waiting = inbox.take().expect("the new receiver");
    assert_eq!(waiting.try_recv(), Some(Notice::Reload));
}
