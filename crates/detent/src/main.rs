//! `detent`: the desktop's Settings app, one window on Blitz.

mod cli;
mod dirs;

use clap::Parser;
use detent_ui::{App, Env};
use ds_blitz::{AppConfig, AppId, Decorations, launch};
use ds_settings::SystemPrefsSource;
use std::process::ExitCode;

/// The window's size in logical pixels.
const WIDTH: u32 = 900;
const HEIGHT: u32 = 640;

fn main() -> ExitCode {
    let opening = cli::Cli::parse().opening();
    let Some(config_home) = dirs::config_home() else {
        eprintln!("detent: neither XDG_CONFIG_HOME nor HOME is set, nowhere to keep settings");
        return ExitCode::FAILURE;
    };
    let env = Env::discover(
        &dirs::schema_dirs(),
        config_home,
        SystemPrefsSource::Portal,
        opening,
    );
    for warning in &env.warnings {
        eprintln!("detent: {warning}");
    }
    let config = AppConfig::new("Settings", WIDTH, HEIGHT)
        .with_app_id(AppId("detent".to_owned()))
        .with_decorations(Decorations::Client)
        .with_context(env);
    launch(App, config);
    ExitCode::SUCCESS
}
