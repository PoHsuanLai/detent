//! `detent`: the desktop's Settings app, one window on Blitz.

mod accounts;
mod cli;
mod dev_callers;
mod dirs;
mod service;

use accounts::LiveAccounts;
use clap::Parser;
use cli::Run;
use detent_files::{FirstRunMark, FirstRunStanding, KeymapFiles, find_agents_program};
use detent_intents::{
    ServeFault, SettingsProvider, hand_over_session, search_entries, serve_session,
};
use detent_model::{Catalogue, PageSlug, discover, inbox, is_assistant_setup};
use detent_system::bus::BusSource;
use detent_ui::{App, Env, FirstRunApp, Homes, Opening};
use ds_blitz::{
    AppConfig, AppId, Decorations, FileRoots, LaunchError, WindowSize, enter_runtime, launch,
};
use ds_settings::SystemPrefsSource;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

/// The window's size in logical pixels.
const WIDTH: u32 = 900;
const HEIGHT: u32 = 640;

/// The first-run window's.
const FIRST_RUN_WIDTH: u32 = 440;
const FIRST_RUN_HEIGHT: u32 = 520;

/// The window of the assistant's first-run step: short, like the welcome window.
fn window_size(wanted: Option<&PageSlug>) -> WindowSize {
    match wanted {
        Some(slug) if is_assistant_setup(slug) => {
            WindowSize::new(FIRST_RUN_WIDTH, FIRST_RUN_HEIGHT)
        }
        Some(_) | None => WindowSize::new(WIDTH, HEIGHT),
    }
}

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    if let Err(code) = dev_callers::take_up(cli.dev_callers()) {
        return code;
    }
    match cli.run() {
        Run::Settings(opening) => settings(opening),
        Run::Setup => first_run(),
        Run::Service => service::run(),
        Run::SearchEntries { file, schemas } => write_search_entries(&file, schemas),
    }
}

/// Writes the launcher's search entries for the pages and keys of the schemas in `schemas` (the
/// data directories' when none are named) to `file`, for the installer. It reads no settings and
/// opens no window, so it needs no home directory.
fn write_search_entries(file: &Path, schemas: Vec<PathBuf>) -> ExitCode {
    let dirs = if schemas.is_empty() {
        dirs::schema_dirs()
    } else {
        schemas
    };
    let found = discover(&dirs);
    for warning in &found.warnings {
        eprintln!("detent: {warning}");
    }
    let text = search_entries(&Catalogue::build(&found.programs));
    match std::fs::write(file, text) {
        Ok(()) => ExitCode::SUCCESS,
        Err(fault) => {
            eprintln!("detent: cannot write {}: {fault}", file.display());
            ExitCode::FAILURE
        }
    }
}

/// The environment both windows read, from the person's directories.
fn environment(opening: Opening) -> Result<Env, ExitCode> {
    let (Some(config_home), Some(data_home), Some(state_home), Some(cache_home)) = (
        dirs::config_home(),
        dirs::data_home(),
        dirs::state_home(),
        dirs::cache_home(),
    ) else {
        eprintln!("detent: neither XDG_CONFIG_HOME nor HOME is set, nowhere to keep settings");
        return Err(ExitCode::FAILURE);
    };
    let path = std::env::var("PATH").unwrap_or_default();
    let env = Env::discover(
        &dirs::schema_dirs(),
        Homes {
            config: config_home,
            data: data_home,
            state: state_home,
            cache: cache_home,
        },
        SystemPrefsSource::Portal,
        opening,
        BusSource::Session,
    )
    .with_keymap(KeymapFiles::standard())
    .with_application_dirs(dirs::application_dirs())
    .with_intent_dirs(dirs::intent_dirs())
    .with_system_data_dirs(dirs::system_data_dirs())
    .with_programs_path(path.clone())
    .with_agents_program(find_agents_program(
        std::env::current_exe().ok().as_deref(),
        &path,
    ));
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
    let wanted = opening.page.clone();
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
        outbox,
        Arc::new(LiveAccounts(BusSource::Session)),
    );
    // Held until the process ends: dropping the connection takes the service off the bus.
    let _service = match provider {
        Ok(provider) => match enter_runtime_block(serve_session(provider)) {
            Ok(connection) => Some(connection),
            Err(ServeFault::Running) => return hand_over_to_running(wanted),
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
    // The folders the Wallpaper page draws pictures from; the window reads no other `file:` URL.
    let roots = FileRoots::new(env.pictures.roots());
    let config = AppConfig::new("Settings", window_size(wanted.as_ref()))
        .with_app_id(AppId("org.quire.Settings".to_owned()))
        .with_decorations(Decorations::Client)
        .with_file_roots(roots)
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
/// was started with) and leave, so there is never a second window. `page` is the page this
/// process was asked to open, which the running Settings shows. The token and the page go only to
/// an owner of the name that holds the Settings role; to any other program nothing is sent and this
/// process fails with the reason (DETENT-16).
fn hand_over_to_running(page: Option<PageSlug>) -> ExitCode {
    let token = std::env::var("XDG_ACTIVATION_TOKEN")
        .ok()
        .filter(|token| !token.is_empty());
    let handed = enter_runtime_block(hand_over_session(token, page));
    match handed {
        Ok(()) => ExitCode::SUCCESS,
        Err(fault) => {
            eprintln!("detent: could not hand over to the Settings that is running: {fault}");
            ExitCode::FAILURE
        }
    }
}
