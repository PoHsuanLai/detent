//! A write that did not happen, and a file that could not be used.

use ds_helpers::{ReadError, UpdateError, WriteError};
use std::path::{Path, PathBuf};

/// What stopped a save. The messages are for the person (they appear in a toast): plain words,
/// the file named, no protocol or format names.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FilesError {
    /// A directory or file could not be written.
    #[error("Settings couldn't save {path}: {source}")]
    Io {
        /// Which path.
        path: PathBuf,
        /// Why.
        source: std::io::Error,
    },
    /// The file on disk has a mistake in it: writing would replace what the person typed.
    #[error(
        "{path} has a mistake in it, so Settings left it as it is. Fix it by hand first ({reason})."
    )]
    Garbled {
        /// Which file.
        path: PathBuf,
        /// What the parser said.
        reason: String,
    },
    /// The file is there and could not be read (no permission, not text, not an ordinary file,
    /// far too large): it is not taken for an empty one, and nothing is written over it.
    #[error("Settings couldn't read {path} ({reason}), so it left the file as it is.")]
    Unreadable {
        /// Which file.
        path: PathBuf,
        /// Why, in a short phrase.
        reason: String,
    },
    /// The file was changed by another program since it was read: nothing is written over that.
    #[error(
        "Settings couldn't save {path} because another program changed it just now, so it left \
         the file as it is. Try again."
    )]
    Changed {
        /// Which file.
        path: PathBuf,
    },
}

impl From<WriteError> for FilesError {
    fn from(error: WriteError) -> FilesError {
        FilesError::Io {
            path: error.path,
            source: error.source,
        }
    }
}

impl FilesError {
    /// The error of an update of the file at `path`: the file's own refusal as it was, a read
    /// that failed as unreadable, a write that failed as a write.
    pub(crate) fn of_update(path: &Path, error: UpdateError<FilesError>) -> FilesError {
        match error {
            UpdateError::Read(error) => FilesError::unreadable(path, &error),
            UpdateError::Edit(error) => error,
            UpdateError::Write(error) => error.into(),
            // `UpdateError` may grow; a case this does not know is a failed write.
            _ => FilesError::Io {
                path: path.to_path_buf(),
                source: std::io::Error::other("the file could not be updated"),
            },
        }
    }

    /// The file at `path` could not be read, because of `error`.
    pub(crate) fn unreadable(path: &Path, error: &ReadError) -> FilesError {
        FilesError::Unreadable {
            path: path.to_path_buf(),
            reason: reason_of(error),
        }
    }
}

/// Why a read failed, as a short phrase for a sentence.
pub(crate) fn reason_of(error: &ReadError) -> String {
    match error {
        ReadError::NotFound { .. } => "it isn't there".to_owned(),
        ReadError::NotRegular { .. } => "it isn't an ordinary file".to_owned(),
        ReadError::TooBig { .. } => "it is far larger than a settings file".to_owned(),
        ReadError::NotText { .. } => "it isn't plain text".to_owned(),
        ReadError::Io { source, .. } => source.to_string(),
        other => other.to_string(),
    }
}
