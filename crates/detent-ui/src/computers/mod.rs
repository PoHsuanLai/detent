//! "Your computers": the computers the assistant may borrow models from (with Remove and an Add
//! list of the Tailscale computers that lend), and the computers asking to borrow this one's.

mod io;
mod rows;
mod view;

pub(crate) use view::Computers;
