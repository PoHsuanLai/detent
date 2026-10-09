//! The mark that the first run happened: pending until recorded, then done for good.

use detent_files::{FirstRunMark, FirstRunStanding, StateHome};
use detent_testkit::Scratch;

#[test]
fn a_first_run_is_pending_until_it_is_recorded_and_then_done_for_good() {
    let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "first-run-mark");
    let mark = FirstRunMark::under(&StateHome(scratch.join("state")));
    assert_eq!(mark.standing(), FirstRunStanding::Pending);
    mark.record().expect("recorded");
    assert_eq!(mark.standing(), FirstRunStanding::Done);
    mark.record().expect("recording twice is harmless");
    assert_eq!(mark.standing(), FirstRunStanding::Done);
    let other = FirstRunMark::under(&StateHome(scratch.join("elsewhere")));
    assert_eq!(
        other.standing(),
        FirstRunStanding::Pending,
        "another home has its own mark"
    );
}
