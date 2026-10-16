//! A peer that says nothing is given up on at its deadline.

use crate::deadline::{Deadline, answered, peer};
use std::time::Duration;

const SHORT: Deadline = Deadline::of(Duration::from_millis(30));

#[tokio::test]
async fn a_call_that_never_returns_ends_at_its_deadline() {
    let never = answered(SHORT, peer::ACCOUNTS, std::future::pending::<()>()).await;
    assert_eq!(never, None);
}

#[tokio::test]
async fn a_call_that_returns_is_not_cut_short() {
    let quick = answered(SHORT, peer::ACCOUNTS, async { 7 }).await;
    assert_eq!(quick, Some(7));
}
