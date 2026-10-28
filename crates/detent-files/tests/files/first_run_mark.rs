//! The mark that the first run happened: pending until recorded, then done for good.

use detent_files::{FirstRunMark, StateHome};
use detent_testkit::Scratch;

#[test]
fn a_first_run_is_pending_until_it_is_recorded_and_then_done_for_good() {
    let scratch = Scratch::under(env!("CARGO_TARGET_TMPDIR"), "first-run-mark");
    let mark = FirstRunMark::under(&StateHome(scratch.join("state")));
    assert!(!mark.done());
    mark.record().expect("recorded");
    assert!(mark.done());
    mark.record().expect("recording twice is harmless");
    assert!(mark.done());
    let other = FirstRunMark::under(&StateHome(scratch.join("elsewhere")));
    assert!(!other.done(), "another home has its own mark");
}
