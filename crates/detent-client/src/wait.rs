//! What waiting on another process is made of: the deadlines every call runs under, the names of
//! who is waited on, and the ticks that say "look again". They live in `detent-system`, because
//! its links return them; the client is where a window reaches them.

pub use detent_system::{Deadline, Gone, Ticker, Ticks, answered, peer};
