//! `cargo run -p detent-model --example write_quire_schema -- <dir>`: writes
//! `quire.settings.toml`, the schema of `quire/appearance.toml`, the way `sill --write-schema`
//! writes sill's. quire ships no command that does it, so the fixtures script uses this.

use ds_settings::schema::{AppId, FilePath, Schema, SettingsSchema};
use ds_settings::{AppearanceSettings, IconsSettings};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: write_quire_schema <dir>");
        return ExitCode::FAILURE;
    };
    let schema = Schema {
        app: AppId("quire".to_owned()),
        file: FilePath("quire/appearance.toml".to_owned()),
        version: 1,
        key: [AppearanceSettings::schema(), IconsSettings::schema()]
            .into_iter()
            .flat_map(|schema| schema.key)
            .collect(),
    };
    match schema.write_to(Path::new(&dir)) {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("write_quire_schema: {error}");
            ExitCode::FAILURE
        }
    }
}
