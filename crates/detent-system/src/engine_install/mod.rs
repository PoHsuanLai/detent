//! What the engine sheet does to the computer, behind one handle that the environment carries, so
//! a test hands in a desk that writes down what it was asked and never touches the real system.

mod bus;

pub use bus::OnEngines;

use detent_model::{EngineRow, InstallScope, Work};
use std::fmt::Debug;

/// The things the sheet asks for.
pub trait Desk: Debug + Send + Sync {
    /// The engines now: where each stands and what can be done about it.
    fn rows(&self) -> Work<'_, Vec<EngineRow>>;
    /// Starts downloading `engine` for `scope`; answers at once, the rows follow it. `Err` is the
    /// reason, in plain words.
    fn install(&self, engine: &str, scope: InstallScope) -> Result<(), String>;
    /// Stops the download of `engine`.
    fn cancel(&self, engine: &str);
    /// Asks the person for a folder; `None` when the chooser was closed or there is none.
    fn choose_folder(&self) -> Work<'_, Option<String>>;
    /// Counts `folder` as `engine`'s. `Err` is the reason, in plain words.
    fn point(&self, engine: &str, folder: &str) -> Work<'_, Result<(), String>>;
    /// Restarts the assistant so it finds what was installed.
    fn restart(&self) -> Work<'_, ()>;
}

/// A desk with no engines: nothing is listed and nothing is downloaded.
#[derive(Debug, Clone, Copy)]
pub struct Quiet;

impl Desk for Quiet {
    fn rows(&self) -> Work<'_, Vec<EngineRow>> {
        Box::pin(async { Vec::new() })
    }

    fn install(&self, _engine: &str, _scope: InstallScope) -> Result<(), String> {
        Err("Nothing can be downloaded here.".to_owned())
    }

    fn cancel(&self, _engine: &str) {}

    fn choose_folder(&self) -> Work<'_, Option<String>> {
        Box::pin(async { None })
    }

    fn point(&self, _engine: &str, _folder: &str) -> Work<'_, Result<(), String>> {
        Box::pin(async { Err("Nothing can be chosen here.".to_owned()) })
    }

    fn restart(&self) -> Work<'_, ()> {
        Box::pin(async {})
    }
}
