//! "Your computers": the computers the assistant may borrow models from (their rows are drawn by
//! the Assistants block, with Remove), an Add list of the Tailscale computers that lend, and the
//! computers asking to borrow this one's.

mod io;
mod server_sheet;
mod view;

pub(crate) use io::{inferd, relay};
pub(crate) use view::{Computers, use_computers};
