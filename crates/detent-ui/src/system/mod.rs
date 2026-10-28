//! What the system pages share: the row they show when there is nothing to show, and the state of
//! a row while the page is busy.

mod row_work;
mod unavailable;

pub(crate) use row_work::RowWork;
pub(crate) use unavailable::{Absent, Unavailable};
