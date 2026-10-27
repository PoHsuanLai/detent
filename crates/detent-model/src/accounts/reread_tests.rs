//! Reading the module again: a failed read keeps the list, a lost module recovers, reads queue.

use super::tests::{apply, described, ready, state_of};
use super::*;
use crate::Absence;

fn failed() -> AccountsInput {
    AccountsInput::Reread(ModuleId::Accountd, Err(Absence::Failed))
}

#[test]
fn a_failed_reread_keeps_the_rows_and_tries_again() {
    let before = state_of(ready());
    let (state, effects) = apply(ready(), [failed()]);
    assert_eq!(state_of(state).pane, before.pane);
    assert_eq!(effects, [AccountsEffect::Retry(ModuleId::Accountd)]);
}

#[test]
fn an_unavailable_pane_reads_again_on_a_change_and_shows_the_list_when_found() {
    let gone = AccountsState::Unavailable(Absence::Failed, Sources::default());
    let (state, effects) = apply(gone, [AccountsInput::Changed(ModuleId::Accountd)]);
    assert_eq!(effects, [AccountsEffect::Reload(ModuleId::Accountd)]);
    let (state, _) = apply(
        state,
        [AccountsInput::Found(ModuleId::Accountd, Ok(described()))],
    );
    assert_eq!(state, ready());
}

#[test]
fn an_unavailable_pane_stays_so_when_the_read_fails_again() {
    let gone = AccountsState::Unavailable(Absence::NotRunning, Sources::default());
    let (state, effects) = apply(gone.clone(), [failed()]);
    assert_eq!(state, gone);
    assert!(effects.is_empty());
}

#[test]
fn a_burst_of_asks_during_one_read_gives_exactly_one_more() {
    for burst in [0, 1, 2, 7] {
        let (mut reads, first) = Reads::default().ask();
        let mut started = vec![first];
        for _ in 0..burst {
            let (next, turn) = reads.ask();
            reads = next;
            started.push(turn);
        }
        let (reads, again) = reads.finish();
        let (idle, last) = reads.finish();
        let more = [again, last]
            .into_iter()
            .filter(|t| *t == ReadTurn::Read)
            .count();
        let reads_started = started.iter().filter(|t| **t == ReadTurn::Read).count();
        assert_eq!(reads_started, 1, "burst of {burst}");
        assert_eq!(more, usize::from(burst > 0), "burst of {burst}");
        assert_eq!(idle, Reads::Idle);
    }
}
