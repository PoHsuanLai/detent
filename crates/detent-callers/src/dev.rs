//! A person's own rows (feature `dev-callers`, which only the `detent` binary's feature of the same
//! name turns on): the stand-ins a checkout builds, named in a file the person owns, are let in as the
//! services they stand for, at `Proof::Dev`, below every real proof. A release build has none of this:
//! the feature, the option and peephole's `dev-rows` are all absent.

use peephole_bus::peephole::{LoadError, RealProc, TableFiles};
use peephole_bus::peephole_core::{Table, Uid};
use std::path::Path;
use std::sync::OnceLock;

/// The rows `detent --dev-callers` named, read once at start and merged after the desktop's table by
/// every [`desktop_peers`](crate::desktop_peers).
static ROWS: OnceLock<Table> = OnceLock::new();

/// Why the person's rows could not be taken up.
#[derive(Debug, thiserror::Error)]
pub enum DevRowsFault {
    /// The file is not the person's own plain file, or it does not parse.
    #[error("{0}")]
    Load(#[from] LoadError),
    /// Rows were already taken up in this process.
    #[error("the developer rows were already read")]
    Twice,
}

/// The rows in `path`: a plain file of this user's (or root's) that group and others cannot write,
/// within peephole's size cap, every row a developer row. Anything else is an error.
pub fn load_dev_rows(path: &Path) -> Result<Table, LoadError> {
    let person = Uid::new(rustix::process::getuid().as_raw());
    TableFiles::load_dev(&RealProc::system(), path, person)
}

/// Reads `path` and makes its rows part of every [`desktop_peers`](crate::desktop_peers) from now on.
/// Call it once, at start, before any peers are made.
pub fn use_dev_rows(path: &Path) -> Result<(), DevRowsFault> {
    let rows = load_dev_rows(path)?;
    ROWS.set(rows).map_err(|_| DevRowsFault::Twice)
}

/// The shipped table with the person's rows after it (a row never takes a unit the shipped table
/// names); the shipped table alone when no rows were taken up.
pub(crate) fn with_dev_rows(shipped: Table) -> Table {
    match ROWS.get() {
        Some(rows) => shipped.merge(rows.clone()),
        None => shipped,
    }
}
