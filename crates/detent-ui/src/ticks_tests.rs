//! Ticks cannot pile up, and end their relays when dropped.

use crate::ticks::{Gone, Ticks};
use std::sync::Arc;
use std::time::Duration;

/// Longer than a tick takes to arrive on a test's single thread, short enough not to be felt.
const WAIT: Duration = Duration::from_millis(100);

#[tokio::test]
async fn a_thousand_ticks_while_nobody_listens_are_one() {
    let (ticker, mut ticks) = Ticks::channel();
    for _ in 0..1000 {
        assert_eq!(ticker.tick(), Ok(()));
    }
    assert_eq!(ticks.recv().await, Some(()));
    let another = tokio::time::timeout(WAIT, ticks.recv()).await;
    assert!(another.is_err(), "the flood left a second tick behind");
}

#[tokio::test]
async fn a_ticker_whose_listener_has_gone_says_so() {
    let (ticker, ticks) = Ticks::channel();
    drop(ticks);
    assert_eq!(ticker.tick(), Err(Gone));
}

#[tokio::test]
async fn every_producer_gone_ends_the_ticks() {
    let (ticker, mut ticks) = Ticks::channel();
    drop(ticker);
    assert_eq!(ticks.recv().await, None);
}

#[tokio::test]
async fn dropping_the_ticks_ends_the_relays_that_fed_them() {
    let alive = Arc::new(());
    let held = Arc::clone(&alive);
    let (_ticker, ticks) = Ticks::channel();
    let ticks = ticks.with_relay(tokio::spawn(async move {
        let _held = held;
        std::future::pending::<()>().await;
    }));
    tokio::task::yield_now().await;
    assert_eq!(Arc::strong_count(&alive), 2, "the relay is running");
    drop(ticks);
    for _ in 0..50 {
        if Arc::strong_count(&alive) == 1 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(Arc::strong_count(&alive), 1, "the relay outlived the page");
}
