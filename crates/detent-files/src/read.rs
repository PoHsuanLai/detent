//! The one way this crate reads a person's file as text.

use ds_helpers::{ReadCap, ReadError, read_text_capped};
use std::path::Path;

/// The most a settings file may hold: the cap quire's own settings reader uses. Hand-edited
/// settings are a few KiB; a megabyte is already a mistake, and a bigger file (or a pipe or a
/// device pointed at by a link) must not stall the window, which reads it on its own task.
pub(crate) const SETTINGS_CAP: ReadCap = ReadCap::mib(1);

/// The text of the file at `path`: `None` when there is no such file, an error for every other
/// reason a file cannot be read. A file that exists but cannot be read is never an empty one,
/// so nothing is ever written over it as if it were new.
pub(crate) fn text_of(path: &Path) -> Result<Option<String>, ReadError> {
    match read_text_capped(path, SETTINGS_CAP) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.is_missing() => Ok(None),
        Err(error) => Err(error),
    }
}
