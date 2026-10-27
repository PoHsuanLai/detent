//! llama.cpp on this computer, as the assistant lists it, and the assistant's restart in the
//! person's own session. Looking never starts the assistant: with it not running there is nothing
//! to say, and opening Settings must not start it. Restarting is the person's word.

mod bus;

pub use bus::{OnBus, restart};

use detent_model::{LlamaCpp, Restarted, Work};
use std::fmt::Debug;

/// What the engines note asks of the assistant.
pub trait EnginesLink: Debug + Send + Sync {
    /// Where llama.cpp stands in the engines the assistant lists now; unknown when the assistant
    /// is not running, not answering, or not allowing Settings to ask.
    fn look(&self) -> Work<'_, LlamaCpp>;
    /// Restarts the assistant in the person's own session, and answers when it is up again.
    fn restart(&self) -> Work<'_, Restarted>;
}
