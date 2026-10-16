//! Finding the schemas programs ship (design/22 section 9.2 and 9.3 item 1).

use crate::config_file::ConfigFile;
use crate::origin::{OriginRule, SchemaOrigin};
use crate::warning::{StaleReason, Warning};
use ds_settings::schema::{AgentSetting, AppId, ForeignTables, KeySpec, Schema};
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
    /// Its keys, in declaration order. Which of them an agent may set is each key's own
    /// `KeySpec::agent`, kept only for a schema from a [`SchemaOrigin::System`] folder; every key
    /// of a schema from a folder the person can write reads as `HandsOff`.
    pub keys: Vec<KeySpec>,
    /// The file's tables that are the program's own data, not settings.
    pub foreign: ForeignTables,
}

/// What a scan found: the usable schemas and the ones left out.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Discovery {
    /// Usable schemas, in directory order and then file name order.
    pub programs: Vec<ProgramSchema>,
    /// Stale schemas, one per skipped file.
    pub warnings: Vec<Warning>,
}

/// Every schema under `dirs`, judged by who owns the folders ([`OriginRule::Ownership`]).
pub fn discover(dirs: &[PathBuf]) -> Discovery {
    discover_under(dirs, &OriginRule::Ownership)
}

/// Every schema under `dirs` (`ds_settings::schema::data_dirs`: the system's directories, then
/// `$XDG_DATA_HOME`'s, in the order of trust), the folders judged by `rule`. `ds_settings` admits
/// a schema only for its own program and location (the first one found for a program wins, so the
/// user's directory never replaces an installed schema) and reports the rest on stderr. One it
/// admits that this app cannot use (an unknown version, a `file` that is not
/// `<app-dir>/<name>.toml`) is skipped and returned as a warning.
///
/// A key's `agent` mark is that of the schema admitted here, and is honoured only when its folder
/// is [`SchemaOrigin::System`]: a schema from a folder the person can write still describes its
/// program, but none of its keys is agent-settable. `ds_settings` does not say which folder a
/// schema came from, so each folder is scanned on its own and a program already found in an
/// earlier folder is not taken again.
pub fn discover_under(dirs: &[PathBuf], rule: &OriginRule) -> Discovery {
    let mut seen: Vec<AppId> = Vec::new();
    let mut found = Discovery::default();
    for dir in dirs {
        let origin = rule.judge(dir);
        for schema in ds_settings::schema::discover(std::slice::from_ref(dir)) {
            if !seen.contains(&schema.app) {
                seen.push(schema.app.clone());
                found = admit(found, schema, origin);
            }
        }
    }
    found
}

/// `key`, as an agent may use it from a schema of `origin`.
fn honoured(origin: SchemaOrigin, key: KeySpec) -> KeySpec {
    match origin {
        SchemaOrigin::System => key,
        SchemaOrigin::PersonWritable => KeySpec {
            agent: AgentSetting::HandsOff,
            ..key
        },
    }
}

fn admit(mut found: Discovery, schema: Schema, origin: SchemaOrigin) -> Discovery {
    let stale = |reason| Warning {
        app: schema.app.clone(),
        reason,
    };
    let checked = if schema.version != SUPPORTED_VERSION {
        Err(StaleReason::UnsupportedVersion(schema.version))
    } else {
        ConfigFile::parse(&schema.file)
    };
    match checked {
        Ok(file) => found.programs.push(ProgramSchema {
            app: schema.app,
            file,
            keys: schema
                .key
                .into_iter()
                .map(|key| honoured(origin, key))
                .collect(),
            foreign: schema.foreign,
        }),
        Err(reason) => found.warnings.push(stale(reason)),
    }
    found
}
