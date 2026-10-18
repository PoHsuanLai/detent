//! Where a live pane stands: not read yet, not there, or read. The one wrapper every pane that
//! reads a service shares; each pane keeps its own `Ready` data, input, effects and step.

use crate::Absence;

/// Where a live pane stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Live<R> {
    /// Looking for the service.
    #[default]
    Looking,
    /// The service cannot be used, and why.
    Unavailable(Absence),
    /// The service answered.
    Ready(R),
}

impl<R> Live<R> {
    /// The state after a read: a good read refreshes the ready state (`refresh`, given the state
    /// and what was read) or starts one (`fresh`); a failed read makes the pane unavailable.
    pub fn found<D>(
        self,
        read: Result<D, Absence>,
        fresh: impl FnOnce(D) -> R,
        refresh: impl FnOnce(R, D) -> R,
    ) -> Live<R> {
        match (self, read) {
            (Live::Ready(ready), Ok(read)) => Live::Ready(refresh(ready, read)),
            (_, Ok(read)) => Live::Ready(fresh(read)),
            (_, Err(absence)) => Live::Unavailable(absence),
        }
    }

    /// The state after `step` on the ready state, and its effects; a pane that is not ready
    /// stays as it is and has none.
    pub fn inner<E>(self, step: impl FnOnce(R) -> (R, Vec<E>)) -> (Live<R>, Vec<E>) {
        match self {
            Live::Ready(ready) => {
                let (ready, effects) = step(ready);
                (Live::Ready(ready), effects)
            }
            other => (other, Vec::new()),
        }
    }
}
