//! A file's problems are said once, not on every read (audit DETENT-18). Every write the intents
//! provider makes reads each file again, and a hand edit's unknown keys would otherwise fill the
//! journal once per agent call.

use std::collections::BTreeSet;
use std::sync::{Mutex, PoisonError};

/// The lines said by the last read.
#[derive(Debug)]
pub(crate) struct Reported(Mutex<BTreeSet<String>>);

impl Reported {
    /// Nothing said yet.
    pub(crate) const fn new() -> Reported {
        Reported(Mutex::new(BTreeSet::new()))
    }

    /// Of `lines` (every problem the latest read found), the ones the read before did not find:
    /// those are said now. A problem that was fixed and comes back is said again.
    pub(crate) fn fresh(&self, lines: BTreeSet<String>) -> Vec<String> {
        let mut said = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let new = lines.difference(&said).cloned().collect();
        *said = lines;
        new
    }
}

#[cfg(test)]
mod tests {
    use super::Reported;
    use std::collections::BTreeSet;

    fn lines(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|line| (*line).to_owned()).collect()
    }

    #[test]
    fn a_problem_is_said_once_until_it_goes_and_comes_back() {
        let reported = Reported::new();
        assert_eq!(reported.fresh(lines(&["a"])), ["a"]);
        assert_eq!(reported.fresh(lines(&["a"])), Vec::<String>::new());
        assert_eq!(reported.fresh(lines(&["a", "b"])), ["b"]);
        assert_eq!(reported.fresh(lines(&[])), Vec::<String>::new());
        assert_eq!(reported.fresh(lines(&["a"])), ["a"], "it came back");
    }
}
