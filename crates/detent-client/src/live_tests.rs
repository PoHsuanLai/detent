//! A live value tells the window each state and each notice, in the order they happen.

use crate::live::{Heard, Live};
use crate::notice::Notice;

fn add(state: u32, input: u32) -> (u32, Vec<u32>) {
    (state + input, vec![input])
}

#[tokio::test]
async fn a_step_changes_the_state_at_once_and_tells_the_window() {
    let live = Live::<u32>::default();
    let effects = live.step(add, 2);
    assert_eq!(effects, [2]);
    assert_eq!(live.state(), 2);
    let events = live.events();
    assert!(matches!(events.next().await, Some(Heard::State(2))));
}

#[tokio::test]
async fn states_and_notices_arrive_in_the_order_they_were_made() {
    let live = Live::<u32>::default();
    live.update(|state| state + 1);
    live.notice(Notice::PoolRefused);
    live.update(|state| state + 1);
    let events = live.events();
    assert!(matches!(events.next().await, Some(Heard::State(1))));
    assert!(matches!(
        events.next().await,
        Some(Heard::Notice(Notice::PoolRefused))
    ));
    assert!(matches!(events.next().await, Some(Heard::State(2))));
}
