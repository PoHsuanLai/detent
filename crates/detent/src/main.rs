//! `detent`: the desktop's Settings app, one window on Blitz.

mod accounts;
mod cli;
mod dirs;
mod link;

use accounts::LiveAccounts;
use clap::Parser;
use cli::Run;
use detent_files::{FirstRunMark, FirstRunStanding};
use detent_intents::{SettingsProvider, serve_session};
use detent_ui::{App, BusSource, Env, FirstRunApp, Opening, inbox};
use ds_blitz::{AppConfig, AppId, Decorations, WindowSize, enter_runtime, launch};
use ds_settings::SystemPrefsSource;
use link::WindowLink;
use std::process::ExitCode;
use std::sync::Arc;

/// The window's size in logical pixels.
const WIDTH: u32 = 900;
const HEIGHT: u32 = 640;

/// The first-run window's.
const FIRST_RUN_WIDTH: u32 = 440;
const FIRST_RUN_HEIGHT: u32 = 520;

fn main() -> ExitCode {
    match cli::Cli::parse().run() {
        Run::Settings(opening) => settings(opening),
        Run::FirstRun => first_run(),
    }
}

/// The environment both windows read, from the person's directories.
fn environment(opening: Opening) -> Result<Env, ExitCode> {
    let (Some(config_home), Some(state_home)) = (dirs::config_home(), dirs::state_home()) else {
        eprintln!("detent: neither XDG_CONFIG_HOME nor HOME is set, nowhere to keep settings");
        return Err(ExitCode::FAILURE);
    };
    let env = Env::discover(
        &dirs::schema_dirs(),
        config_home,
        &state_home,
        SystemPrefsSource::Portal,
        opening,
        BusSource::Session,
    );
    for warning in &env.warnings {
        eprintln!("detent: {warning}");
    }
    Ok(env)
}

/// The one-time welcome window; a run after it has been shown exits at once, before anything
/// else is read or opened.
fn first_run() -> ExitCode {
    let Some(state_home) = dirs::state_home() else {
        eprintln!(
            "detent: neither XDG_STATE_HOME nor HOME is set, nowhere to remember the first run"
        );
        return ExitCode::FAILURE;
    };
    if FirstRunMark::under(&state_home).standing() == FirstRunStanding::Done {
        return ExitCode::SUCCESS;
    }
    let env = match environment(Opening::default()) {
        Ok(env) => env,
        Err(code) => return code,
    };
    let config = AppConfig::new(
        "Welcome",
        WindowSize::new(FIRST_RUN_WIDTH, FIRST_RUN_HEIGHT),
    )
    .with_app_id(AppId("org.quire.Settings".to_owned()))
    .with_decorations(Decorations::Client)
    .with_context(env);
    launch(FirstRunApp, config);
    ExitCode::SUCCESS
}

fn settings(opening: Opening) -> ExitCode {
    let env = match environment(opening) {
        Ok(env) => env,
        Err(code) => return code,
    };
    let (outbox, inbox) = inbox();
    let env = Env { inbox, ..env };
    let config_home = env.config_home.clone();
    // The runtime `launch` will use: the intents service lives on it for the app's life.
    let _runtime = enter_runtime();
    match SettingsProvider::new(
        env.catalogue.clone(),
        config_home,
        Arc::new(WindowLink(outbox)),
        Arc::new(LiveAccounts(BusSource::Session)),
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

/// Answers the desktop's intents (open a page, read and set a key, list the accounts) on the session bus while the
/// window runs. Without a bus, or with another Settings already on it, the window runs without.
async fn serve_intents(provider: SettingsProvider) {
    match serve_session(provider).await {
        // Dropping the connection takes the service off the bus: keep it until the process ends.
        Ok(_connection) => std::future::pending::<()>().await,
        Err(fault) => eprintln!("detent: no intents service: {fault}"),
    }
}
