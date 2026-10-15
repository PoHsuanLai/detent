//! Logging out, so a new version starts: whatever the desktop offers for it, behind one handle the
//! environment carries, so a test hands in a stand-in that only writes down that it was asked.

use super::link::UpdateWork;
use std::fmt;
use std::process::{Command, Stdio};
use std::sync::Arc;

/// What the person reads when logging out could not be started.
const COULD_NOT: &str =
    "Settings could not start logging out. Log out from the power menu of the desktop instead.";

/// What the page needs to start leaving the session.
pub trait Departure: Send + Sync {
    /// Starts logging out; the sentence that says why not when it could not be started.
    fn log_out(&self) -> UpdateWork<'_, Result<(), String>>;
}

/// How the page logs the person out, as the environment carries it.
#[derive(Clone)]
pub struct LogOut(Arc<dyn Departure>);

impl fmt::Debug for LogOut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogOut").finish_non_exhaustive()
    }
}

impl LogOut {
    /// Logging out over `departure`: what a test hands in.
    pub fn over(departure: impl Departure + 'static) -> LogOut {
        LogOut(Arc::new(departure))
    }

    /// The desktop's own power menu, which asks before it logs out.
    pub fn power_menu() -> LogOut {
        LogOut::over(PowerMenu)
    }

    pub(crate) fn start(&self) -> UpdateWork<'_, Result<(), String>> {
        self.0.log_out()
    }
}

/// The shell's power menu, opened by its command.
struct PowerMenu;

fn open_power_menu() -> Result<(), String> {
    let opened = Command::new("sill")
        .args(["power-menu", "open"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match opened {
        Ok(status) if status.success() => Ok(()),
        _ => Err(COULD_NOT.to_owned()),
    }
}

impl Departure for PowerMenu {
    fn log_out(&self) -> UpdateWork<'_, Result<(), String>> {
        Box::pin(async {
            tokio::task::spawn_blocking(open_power_menu)
                .await
                .unwrap_or_else(|_| Err(COULD_NOT.to_owned()))
        })
    }
}
