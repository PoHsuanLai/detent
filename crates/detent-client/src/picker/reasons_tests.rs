//! The feed of reasons is handed over once.

use super::reasons::ReasonFeed;

#[tokio::test]
async fn the_second_taker_of_the_feed_hears_nothing() {
    let (_sender, feed) = ReasonFeed::channel();
    let _first = feed.take();
    let mut second = feed.take();
    assert!(second.next().await.is_none());
}
