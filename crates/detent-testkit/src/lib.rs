//! Test support, std only: a scratch directory that removes itself, the real schemas
//! `scripts/gen-fixtures.sh` wrote into `fixtures/schemas`, and Extensions installed as a package
//! would.

mod extensions;
mod fixtures;
mod scratch;

pub use extensions::{install_file_icons, install_look, install_torn_look, install_your_changes};
pub use fixtures::{fixture_dir, seed_schemas, seed_schemas_settable};
pub use scratch::Scratch;
