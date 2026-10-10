//! `detent`: the desktop's Settings app, one window on Blitz.

mod accounts;
mod cli;
mod dirs;
mod link;

use accounts::LiveAccounts;
use clap::Parser;
use cli::Run;
use detent_files::{FirstRunMark, FirstRunStanding, KeymapFiles};
use detent_intents::{ServeFault, SettingsProvider, hand_over, serve_session};
use detent_ui::{App, BusSource, Env, FirstRunApp, Opening, inbox};
use ds_blitz::{AppConfig, AppId, Decorations, LaunchError, WindowSize, enter_runtime, launch};
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
    let (Some(config_home), Some(data_home), Some(state_home)) =
        (dirs::config_home(), dirs::data_home(), dirs::state_home())
    else {
        eprintln!("detent: neither XDG_CONFIG_HOME nor HOME is set, nowhere to keep settings");
        return Err(ExitCode::FAILURE);
    };
    let env = Env::discover(
        &dirs::schema_dirs(),
        config_home,
        data_home,
        &state_home,
        SystemPrefsSource::Portal,
        opening,
        BusSource::Session,
    )
    .with_keymap(KeymapFiles::standard());
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
    ended(launch(FirstRunApp, config))
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
    let provider = SettingsProvider::new(
        env.catalogue.clone(),
        config_home,
        Arc::new(WindowLink(outbox)),
        Arc::new(LiveAccounts(BusSource::Session)),
    );
    // Held until the process ends: dropping the connection takes the service off the bus.
    let _service = match provider {
        Ok(provider) => match enter_runtime_block(serve_session(provider)) {
            Ok(connection) => Some(connection),
            Err(ServeFault::Running) => return hand_over_to_running(),
            Err(fault) => {
                eprintln!("detent: no intents service: {fault}");
                None
            }
        },
        Err(fault) => {
            eprintln!("detent: no intents service: {fault}");
            None
        }
    };
    let config = AppConfig::new("Settings", WindowSize::new(WIDTH, HEIGHT))
        .with_app_id(AppId("org.quire.Settings".to_owned()))
        .with_decorations(Decorations::Client)
        .with_context(env);
    ended(launch(App, config))
}

/// The exit code of a window that ran to its end: failure, with the reason said, when it could not run.
fn ended(launched: Result<(), LaunchError>) -> ExitCode {
    match launched {
        Ok(()) => ExitCode::SUCCESS,
        Err(fault) => {
            eprintln!("detent: {fault}");
            ExitCode::FAILURE
        }
    }
}

/// Runs `future` to its end on the process-wide runtime, before any window exists.
fn enter_runtime_block<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Handle::current().block_on(future)
}

/// Another Settings is running: ask it to show itself (with the activation token this process
/// was started with) and leave, so there is never a second window.
fn hand_over_to_running() -> ExitCode {
    let token = std::env::var("XDG_ACTIVATION_TOKEN")
        .ok()
        .filter(|token| !token.is_empty());
    let handed = enter_runtime_block(async {
        let connection = zbus::connection::Builder::session()?.build().await?;
        hand_over(&connection, token).await
    });
    match handed {
        Ok(()) => ExitCode::SUCCESS,
        Err(fault) => {
            eprintln!("detent: Settings is running but did not answer: {fault}");
            ExitCode::FAILURE
        }
    }
}
