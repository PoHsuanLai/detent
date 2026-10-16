//! Where a schema came from, and what that lets it say about agents.

use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// The owner of a folder only an administrator can change.
const ROOT: u32 = 0;

/// The mode bits that let a group or anyone else write into a folder.
const GROUP_OR_OTHER_WRITE: u32 = 0o022;

/// Whether the person's own account (or a program running as them) could have written the
/// schemas in a folder. Only a schema from a folder they cannot write may mark a key
/// agent-settable: a schema they can write is theirs to display and to edit by, never a grant to
/// an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaOrigin {
    /// A folder owned by root and writable by nobody else.
    System,
    /// Any other folder, such as `~/.local/share/quire/settings`.
    PersonWritable,
}

impl SchemaOrigin {
    /// The origin of a folder (or any of its parents) owned by `owner` with permission bits
    /// `mode`.
    pub(crate) fn of_owner(owner: u32, mode: u32) -> SchemaOrigin {
        if owner == ROOT && mode & GROUP_OR_OTHER_WRITE == 0 {
            SchemaOrigin::System
        } else {
            SchemaOrigin::PersonWritable
        }
    }

    /// The origin of `dir`: `System` only when the folder and every parent above it (after links
    /// are resolved) are root's and closed to everyone else, since a writable parent could swap
    /// the folder. A folder that cannot be read is the person's.
    pub fn of_dir(dir: &Path) -> SchemaOrigin {
        let Ok(real) = dir.canonicalize() else {
            return SchemaOrigin::PersonWritable;
        };
        let closed = real.ancestors().all(|folder| {
            folder.metadata().is_ok_and(|meta| {
                SchemaOrigin::of_owner(meta.uid(), meta.mode()) == SchemaOrigin::System
            })
        });
        if closed {
            SchemaOrigin::System
        } else {
            SchemaOrigin::PersonWritable
        }
    }
}

/// How a schema folder is judged.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OriginRule {
    /// By who owns it and who may write to it.
    #[default]
    Ownership,
    /// Developer installs and tests only (debug builds): these folders are `System` and every
    /// other folder is `PersonWritable`, whoever owns them. Release builds have no such rule.
    #[cfg(debug_assertions)]
    Declared(Vec<std::path::PathBuf>),
}

impl OriginRule {
    /// A rule that treats exactly `system` as system folders.
    #[cfg(debug_assertions)]
    pub fn declared(system: &[std::path::PathBuf]) -> OriginRule {
        OriginRule::Declared(system.to_vec())
    }

    /// The origin of `dir` under this rule.
    pub fn judge(&self, dir: &Path) -> SchemaOrigin {
        match self {
            OriginRule::Ownership => SchemaOrigin::of_dir(dir),
            #[cfg(debug_assertions)]
            OriginRule::Declared(system) => {
                if system.iter().any(|named| named == dir) {
                    SchemaOrigin::System
                } else {
                    SchemaOrigin::PersonWritable
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SchemaOrigin::{self, PersonWritable, System};

    /// `(case, owner, mode, origin)`.
    const CASES: &[(&str, u32, u32, SchemaOrigin)] = &[
        ("root's, closed", 0, 0o040755, System),
        ("root's, tighter", 0, 0o040700, System),
        ("root's, group-writable", 0, 0o040775, PersonWritable),
        ("root's, other-writable", 0, 0o040757, PersonWritable),
        (
            "root's, sticky and open (like /tmp)",
            0,
            0o041777,
            PersonWritable,
        ),
        ("the person's, closed", 1000, 0o040755, PersonWritable),
        ("another system account's", 1, 0o040755, PersonWritable),
    ];

    #[test]
    fn only_a_root_owned_folder_closed_to_others_is_system() {
        for (name, owner, mode, want) in CASES {
            assert_eq!(SchemaOrigin::of_owner(*owner, *mode), *want, "{name}");
        }
    }
}
