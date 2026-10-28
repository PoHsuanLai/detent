//! A module's reads go one at a time, and the latest ask wins.

use super::feeds::Feed;
use detent_model::Reads;

#[test]
fn a_second_ask_while_one_reads_waits_for_it_and_reads_once_more() {
    let feed = Feed::default();
    assert!(feed.turn(Reads::ask));
    assert!(!feed.turn(Reads::ask));
    assert!(feed.turn(Reads::finish));
    assert!(!feed.turn(Reads::finish));
}
