//! `detent --service`: the intents with no window. The bus starts Settings this way to answer an
//! action (a read, a write, a search-less launcher asking for the pages), and nothing is drawn or
//! focused until an action asks for a window: `settings.page.show`, `settings.page.open`, the
//! freedesktop `Activate`/`ActivateAction` or a later plain `detent` handing over. The window
//! is opened on demand, and the process leaves once no window has been open for [`IDLE`].

use crate::{HEIGHT, WIDTH, ended, enter_runtime_block, environment};
use detent_intents::{ServeFault, SettingsProvider, serve_session};
use detent_model::{Inbox, Notice, Outbox, Receiver, inbox};
use detent_system::bus::BusSource;
use detent_ui::{App, Env, Opening};
use ds_blitz::{
    AppConfig, AppHandle, AppId, Decorations, FileRoots, LastWindowClosed, WindowSize, WindowSpec,
    enter_runtime, launch_idle,
};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

/// How long the service stays up with no window open: long enough that a burst of actions (an
/// agent reading three keys) starts it once, short enough that nothing idles all session.
const IDLE: Duration = Duration::from_secs(120);

/// Whether a notice is something to show. A reload of the files is for a window that exists.
fn wants_window(notice: &Notice) -> bool {
    !matches!(notice, Notice::Reload)
}

/// The window an action opens.
fn window_spec() -> WindowSpec {
    WindowSpec::new("Settings", WindowSize::new(WIDTH, HEIGHT))
        .with_app_id(AppId("org.quire.Settings".to_owned()))
        .with_decorations(Decorations::Client)
}

/// Hands each notice to the open window, or opens one for it. A window that has closed has dropped
/// its receiver, which is how this finds out (`WindowGone`): the next notice that shows something
/// gets a fresh queue in `windows`, which the new window takes, and a window opened for it.
async fn dispatch(mut notices: Receiver, windows: Inbox, app: AppHandle) {
    let mut open: Option<Outbox> = None;
    while let Some(notice) = notices.recv().await {
        if open
            .as_ref()
            .is_some_and(|window| window.send(notice.clone()).is_ok())
        {
            continue;
        }
        open = None;
        if !wants_window(&notice) {
            continue;
        }
        let window = windows.refill();
        if app.open_window(window_spec(), App).is_err() {
            return;
        }
        if window.send(notice).is_ok() {
            open = Some(window);
        }
    }
}

/// Serves the intents with no window until [`IDLE`] passes with none open.
pub fn run() -> ExitCode {
    let windows = Inbox::default();
    let env = match environment(Opening::default()) {
        Ok(env) => Env {
            inbox: windows.clone(),
            ..env
        },
        Err(code) => return code,
    };
    let (outbox, service) = inbox();
    let app = AppHandle::new();
    let _runtime = enter_runtime();
    let provider = match SettingsProvider::new(
        env.catalogue.clone(),
        env.config_home.clone(),
        outbox,
        Arc::new(crate::accounts::LiveAccounts(BusSource::Session)),
    ) {
        Ok(provider) => provider,
        Err(fault) => {
            eprintln!("detent: no intents service: {fault}");
            return ExitCode::FAILURE;
        }
    };
    // Held until the process ends: dropping the connection takes the service off the bus.
    let _connection = match enter_runtime_block(serve_session(provider)) {
        Ok(connection) => connection,
        // Settings is already up and answering: this start has nothing to add.
        Err(ServeFault::Running) => return ExitCode::SUCCESS,
        Err(fault) => {
            eprintln!("detent: no intents service: {fault}");
            return ExitCode::FAILURE;
        }
    };
    let Some(notices) = service.take() else {
        return ExitCode::FAILURE;
    };
    tokio::spawn(dispatch(notices, windows, app.clone()));
    let roots = FileRoots::new(env.pictures.roots());
    let config = AppConfig::new("Settings", WindowSize::new(WIDTH, HEIGHT))
        .with_app_id(AppId("org.quire.Settings".to_owned()))
        .with_decorations(Decorations::Client)
        .with_file_roots(roots)
        .with_handle(app)
        .with_last_window(LastWindowClosed::StayFor(IDLE))
        .with_context(env);
    ended(launch_idle(config))
}

#[cfg(test)]
mod tests {
    use super::wants_window;
    use detent_model::{Notice, PageSlug};

    #[test]
    fn only_a_reload_is_not_worth_a_window() {
        assert!(!wants_window(&Notice::Reload));
        assert!(wants_window(&Notice::Raise(None)));
        assert!(wants_window(&Notice::OpenPage(PageSlug("dock".to_owned()))));
    }
}
