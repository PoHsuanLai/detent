//! The window's inbox cannot be made to grow by whoever calls its activation object.

use crate::inbox::{Notice, inbox};

#[test]
fn a_flood_of_notices_leaves_a_bounded_queue() {
    let (outbox, inbox) = inbox();
    for _ in 0..10_000 {
        assert!(
            outbox.send(Notice::Reload).is_ok(),
            "a full inbox drops, it does not fail"
        );
    }
    let mut waiting = inbox.take().expect("the receiver");
    let mut count = 0;
    while waiting.try_recv().is_ok() {
        count += 1;
    }
    assert!((1..=64).contains(&count), "{count} notices waited");
}

#[test]
fn a_window_that_has_gone_is_told_so() {
    let (outbox, inbox) = inbox();
    drop(inbox.take());
    assert!(outbox.send(Notice::Reload).is_err());
}
