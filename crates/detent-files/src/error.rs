//! A write that did not happen.

use std::path::PathBuf;

/// What stopped a save.
#[derive(Debug, thiserror::Error)]
pub enum FilesError {
    /// A directory or file could not be written or renamed.
    #[error("{path}: {source}")]
    Io {
        /// Which path.
        path: PathBuf,
        /// Why.
        source: std::io::Error,
    },
    /// The file on disk is not valid TOML: writing would replace what the person typed.
    #[error("{path} is not valid TOML, fix it by hand first: {reason}")]
    Garbled {
        /// Which file.
        path: PathBuf,
        /// What the parser said.
        reason: String,
    },
}
