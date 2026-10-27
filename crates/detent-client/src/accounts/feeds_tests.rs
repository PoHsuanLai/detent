//! A module's reads go one at a time, and the latest ask wins.

use super::feeds::Feed;
use detent_model::{ReadTurn, Reads};

#[test]
fn a_second_ask_while_one_reads_waits_for_it_and_reads_once_more() {
    let feed = Feed::default();
    assert_eq!(feed.turn(Reads::ask), ReadTurn::Read);
    assert_eq!(feed.turn(Reads::ask), ReadTurn::Rest);
    assert_eq!(feed.turn(Reads::finish), ReadTurn::Read);
    assert_eq!(feed.turn(Reads::finish), ReadTurn::Rest);
}
