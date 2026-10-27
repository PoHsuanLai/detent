//! A lock that a panicking test does not poison for the next: the fakes' state is plain data.

use std::sync::{Mutex, MutexGuard, PoisonError};

/// `mutex`, locked; a lock a panic left behind is taken as it is.
pub(crate) fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
