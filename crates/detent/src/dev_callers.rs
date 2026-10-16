//! `--dev-callers FILE`, which only a build with the `dev-callers` feature has: the person's own rows
//! are read once, at start, before any program on the bus is judged. A release build has no such
//! option, so there is nothing to take up.

use std::path::Path;
use std::process::ExitCode;

/// Takes up the rows in `path`, when the run was given any. Rows that cannot be read end the start:
/// a stand-in that is silently refused would look like a service that is not running.
#[cfg(feature = "dev-callers")]
pub fn take_up(path: Option<&Path>) -> Result<(), ExitCode> {
    let Some(path) = path else {
        return Ok(());
    };
    detent_callers::use_dev_rows(path).map_err(|fault| {
        eprintln!("detent: --dev-callers {}: {fault}", path.display());
        ExitCode::FAILURE
    })?;
    eprintln!(
        "detent: programs named in {} are let in as the desktop's services, below every real proof (--dev-callers)",
        path.display()
    );
    Ok(())
}

/// A build without the feature has no option, so there is never a file to take up.
#[cfg(not(feature = "dev-callers"))]
pub fn take_up(_path: Option<&Path>) -> Result<(), ExitCode> {
    Ok(())
}
