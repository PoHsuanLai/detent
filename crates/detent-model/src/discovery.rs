//! Finding the schemas programs ship (design/22 section 9.2 and 9.3 item 1).

use crate::agent::{Marks, read_marks};
use crate::config_file::ConfigFile;
use crate::warning::{StaleReason, Warning};
use ds_settings::schema::{AppId, KeyPath, KeySpec, Schema};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The schema `version` this app reads.
const SUPPORTED_VERSION: u16 = 1;

/// One program's schema after the checks: its file is safe and its version is read.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramSchema {
    /// The program.
    pub app: AppId,
    /// The one file all its keys live in.
    pub file: ConfigFile,
    /// Its keys, in declaration order.
    pub keys: Vec<KeySpec>,
    /// The keys its schema marks `agent = "settable"` (see [`crate::AgentSetting`]).
    pub settable: BTreeSet<KeyPath>,
}

/// What a scan found: the usable schemas and the ones left out.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Discovery {
    /// Usable schemas, in directory order and then file name order.
    pub programs: Vec<ProgramSchema>,
    /// Stale schemas, one per skipped file.
    pub warnings: Vec<Warning>,
}

/// Every schema under `dirs` (`$XDG_DATA_HOME` first, then `$XDG_DATA_DIRS`, each with
/// `quire/settings` appended by the caller). A file that does not parse is skipped and reported
/// by `ds_settings`; one that parses but is stale (unknown version, a `file` outside the config
/// home, an app an earlier directory already supplied) is skipped and returned as a warning.
pub fn discover(dirs: &[PathBuf]) -> Discovery {
    let marks = read_marks(dirs);
    ds_settings::schema::discover(dirs)
        .into_iter()
        .fold(Discovery::default(), |found, schema| {
            admit(found, schema, &marks)
        })
}

fn admit(mut found: Discovery, schema: Schema, marks: &Marks) -> Discovery {
    let stale = |reason| Warning {
        app: schema.app.clone(),
        reason,
    };
    let taken = found.programs.iter().any(|p| p.app == schema.app);
    let checked = if schema.version != SUPPORTED_VERSION {
        Err(StaleReason::UnsupportedVersion(schema.version))
    } else if taken {
        Err(StaleReason::Shadowed)
    } else {
        ConfigFile::parse(&schema.file)
    };
    match checked {
        Ok(file) => found.programs.push(ProgramSchema {
            settable: marks.get(&schema.app).cloned().unwrap_or_default(),
            app: schema.app,
            file,
            keys: schema.key,
        }),
        Err(reason) => found.warnings.push(stale(reason)),
    }
    found
}
