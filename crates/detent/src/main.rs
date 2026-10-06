//! `detent`: the desktop's Settings app, one window on Blitz.

mod cli;
mod dirs;
mod link;

use clap::Parser;
use detent_intents::{SettingsProvider, serve_session};
use detent_ui::{App, BusSource, Env, inbox};
use ds_blitz::{AppConfig, AppId, Decorations, WindowSize, enter_runtime, launch};
use ds_settings::SystemPrefsSource;
use link::WindowLink;
use std::process::ExitCode;
use std::sync::Arc;

/// The window's size in logical pixels.
const WIDTH: u32 = 900;
const HEIGHT: u32 = 640;

fn main() -> ExitCode {
    let opening = cli::Cli::parse().opening();
    let Some(config_home) = dirs::config_home() else {
        eprintln!("detent: neither XDG_CONFIG_HOME nor HOME is set, nowhere to keep settings");
        return ExitCode::FAILURE;
    };
    let (outbox, inbox) = inbox();
    let env = Env {
        inbox,
        ..Env::discover(
            &dirs::schema_dirs(),
            config_home.clone(),
            SystemPrefsSource::Portal,
            opening,
            BusSource::Session,
        )
    };
    for warning in &env.warnings {
        eprintln!("detent: {warning}");
    }
    // The runtime `launch` will use: the intents service lives on it for the app's life.
    let _runtime = enter_runtime();
    match SettingsProvider::new(
        env.catalogue.clone(),
        config_home,
        Arc::new(WindowLink(outbox)),
    ) {
        Ok(provider) => {
            tokio::spawn(serve_intents(provider));
        }
        Err(fault) => eprintln!("detent: no intents service: {fault}"),
    }
    let config = AppConfig::new("Settings", WindowSize::new(WIDTH, HEIGHT))
        .with_app_id(AppId("org.quire.Settings".to_owned()))
        .with_decorations(Decorations::Client)
        .with_context(env);
    launch(App, config);
    ExitCode::SUCCESS
}

/// Answers the desktop's intents (open a page, read and set a key) on the session bus while the
/// window runs. Without a bus, or with another Settings already on it, the window runs without.
async fn serve_intents(provider: SettingsProvider) {
    match serve_session(provider).await {
        // Dropping the connection takes the service off the bus: keep it until the process ends.
        Ok(_connection) => std::future::pending::<()>().await,
        Err(fault) => eprintln!("detent: no intents service: {fault}"),
    }
}
