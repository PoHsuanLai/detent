//! Test support, std only: a scratch directory that removes itself, and the real schemas
//! `scripts/gen-fixtures.sh` wrote into `fixtures/schemas`.

mod fixtures;
mod scratch;

pub use fixtures::{fixture_dir, seed_schemas};
pub use scratch::Scratch;
