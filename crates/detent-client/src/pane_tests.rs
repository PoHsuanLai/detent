//! A machine carries out the effects of a step, then the effects of the inputs they lead to, in
//! the order the steps were taken.

use crate::live::{Heard, Live};
use crate::pane::{Machine, Pending, pane};

#[derive(Clone)]
struct Counting {
    live: Live<Vec<u32>>,
}

pane!(Counting, Vec<u32>, follow);

impl Counting {
    async fn follow(&self) {}

    fn send(&self, input: u32) -> Pending {
        self.dispatch(input)
    }
}

impl Machine for Counting {
    type Input = u32;
    type Effect = u32;

    fn step(mut log: Vec<u32>, input: u32) -> (Vec<u32>, Vec<u32>) {
        log.push(input);
        let effects = if input < 3 { vec![input] } else { Vec::new() };
        (log, effects)
    }

    /// Each effect leads to the next number.
    async fn run(&self, effect: u32) -> Vec<u32> {
        vec![effect + 1]
    }
}

#[tokio::test]
async fn every_input_an_effect_leads_to_is_stepped_until_nothing_is_left() {
    let counting = Counting {
        live: Live::default(),
    };
    counting.send(0).await;
    assert_eq!(counting.live.state(), [0, 1, 2, 3]);
}

#[tokio::test]
async fn the_first_step_is_taken_before_the_work_is_polled() {
    let counting = Counting {
        live: Live::default(),
    };
    let work = counting.send(0);
    assert_eq!(counting.live.state(), [0], "the state moved at once");
    work.await;
    let events = counting.live.events();
    assert!(matches!(events.next().await, Some(Heard::State(first)) if first == [0]));
}
