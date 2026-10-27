//! Logging out, so a new version starts: whatever the desktop offers for it, behind one handle the
//! environment carries, so a test hands in a stand-in that only writes down that it was asked.

use detent_model::Work;
use std::fmt::Debug;
use std::process::{Command, Stdio};

/// What the person reads when logging out could not be started.
const COULD_NOT: &str =
    "Settings could not start logging out. Log out from the power menu of the desktop instead.";

/// What the page needs to start leaving the session.
pub trait Departure: Debug + Send + Sync {
    /// Starts logging out; the sentence that says why not when it could not be started.
    fn log_out(&self) -> Work<'_, Result<(), String>>;
}

/// The shell's power menu, opened by its command: the desktop's own, which asks before it logs
/// out.
#[derive(Debug)]
pub struct PowerMenu;

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
    fn log_out(&self) -> Work<'_, Result<(), String>> {
        Box::pin(async {
            tokio::task::spawn_blocking(open_power_menu)
                .await
                .unwrap_or_else(|_| Err(COULD_NOT.to_owned()))
        })
    }
}
