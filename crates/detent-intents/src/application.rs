//! `org.freedesktop.Application` at `/org/quire/Settings`: the Desktop Entry spec's D-Bus
//! activation, so the launcher starts Settings (as its user unit), raises it, or opens it at an
//! account. A second `detent` hands over through the same `Activate`.

use crate::serve::BUS_PEER;
use crate::window::Window;
use detent_model::{AccountTarget, PageSlug};
use ds_peer::{Deadline, within};
use std::collections::HashMap;
use std::sync::Arc;
use zbus::fdo;
use zbus::zvariant::{OwnedValue, Value};

/// The interface served.
pub const APPLICATION_INTERFACE: &str = "org.freedesktop.Application";
/// Where it is served: the bus name `org.quire.Settings` as a path, per the spec.
pub const APPLICATION_PATH: &str = "/org/quire/Settings";
pub(crate) const APP_NAME: &str = "org.quire.Settings";

/// The action that shows the Accounts page at an account.
const OPEN_ACCOUNT: &str = "open-account";
/// The action that shows a page, named by its slug.
const OPEN_PAGE: &str = "open-page";
/// The platform-data key that carries the xdg-activation token.
const TOKEN_KEY: &str = "activation-token";

type PlatformData = HashMap<String, OwnedValue>;

/// The object that answers the launcher.
pub(crate) struct Application {
    window: Arc<dyn Window>,
}

/// The token in `platform_data`, if the caller gave a non-empty one.
fn token_of(platform_data: &PlatformData) -> Option<String> {
    match &**platform_data.get(TOKEN_KEY)? {
        Value::Str(token) if !token.is_empty() => Some(token.to_string()),
        _ => None,
    }
}

/// The account an action's first parameter names: an object path, or the same text as a string.
/// Anything else asks for the list.
fn target_of(parameter: &[OwnedValue]) -> AccountTarget {
    let text = match parameter.first().map(|value| &**value) {
        Some(Value::ObjectPath(path)) => path.as_str().to_owned(),
        Some(Value::Str(text)) => text.to_string(),
        _ => String::new(),
    };
    AccountTarget::of_path(&text)
}

/// The page an action's first parameter names by its slug; none when it names nothing.
fn page_of(parameter: &[OwnedValue]) -> Option<PageSlug> {
    match parameter.first().map(|value| &**value) {
        Some(Value::Str(slug)) if !slug.is_empty() => Some(PageSlug(slug.to_string())),
        _ => None,
    }
}

fn gone() -> fdo::Error {
    fdo::Error::Failed("the Settings window has gone".to_owned())
}

#[zbus::interface(name = "org.freedesktop.Application")]
impl Application {
    /// Show the window and raise it.
    async fn activate(&self, platform_data: PlatformData) -> fdo::Result<()> {
        self.window
            .raise(token_of(&platform_data))
            .map_err(|_| gone())
    }

    /// Settings opens no files.
    async fn open(&self, _uris: Vec<String>, _platform_data: PlatformData) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported(
            "Settings does not open files".to_owned(),
        ))
    }

    /// `open-account` shows the Accounts page at the account named, `open-page` the page whose
    /// slug is named; an unknown action is logged and does nothing.
    async fn activate_action(
        &self,
        action_name: String,
        parameter: Vec<OwnedValue>,
        platform_data: PlatformData,
    ) -> fdo::Result<()> {
        match action_name.as_str() {
            OPEN_ACCOUNT => self
                .window
                .open_account(target_of(&parameter))
                .map_err(|_| gone())?,
            OPEN_PAGE => {
                if let Some(page) = page_of(&parameter) {
                    self.window.open_page(page).map_err(|_| gone())?;
                }
            }
            _ => {
                eprintln!("detent: no action {action_name:?}, ignored");
                return Ok(());
            }
        }
        self.window
            .raise(token_of(&platform_data))
            .map_err(|_| gone())
    }
}

/// Puts the activation object on `connection`.
pub(crate) async fn export_application(
    connection: &zbus::Connection,
    window: Arc<dyn Window>,
) -> Result<(), docket_client::TransportError> {
    connection
        .object_server()
        .at(APPLICATION_PATH, Application { window })
        .await
        .map(|_| ())
        .map_err(|e| docket_client::TransportError::Bus(e.to_string()))
}

/// Asks the running Settings to show itself: what a second `detent` does instead of opening a
/// second window. `token` is the xdg-activation token this process was started with, if any;
/// `page` is the page the second `detent` was asked to open, if any.
pub async fn hand_over(
    connection: &zbus::Connection,
    token: Option<String>,
    page: Option<PageSlug>,
) -> zbus::Result<()> {
    let asked = ask_to_show(connection, token, page);
    within(Deadline::READ, BUS_PEER, asked)
        .await
        .map_err(|silent| zbus::Error::Failure(silent.to_string()))?
}

/// [`hand_over`] on the session bus, connecting first: the whole of it, connecting included,
/// gets the read deadline, so a stuck bus socket ends a second `detent` instead of hanging it.
pub async fn hand_over_session(token: Option<String>, page: Option<PageSlug>) -> zbus::Result<()> {
    let handed = async {
        let connection = zbus::connection::Builder::session()?.build().await?;
        ask_to_show(&connection, token, page).await
    };
    within(Deadline::READ, BUS_PEER, handed)
        .await
        .map_err(|silent| zbus::Error::Failure(silent.to_string()))?
}

async fn ask_to_show(
    connection: &zbus::Connection,
    token: Option<String>,
    page: Option<PageSlug>,
) -> zbus::Result<()> {
    let mut platform_data: HashMap<&str, Value<'_>> = HashMap::new();
    if let Some(token) = &token {
        platform_data.insert(TOKEN_KEY, Value::from(token.as_str()));
    }
    match page {
        None => {
            connection
                .call_method(
                    Some(APP_NAME),
                    APPLICATION_PATH,
                    Some(APPLICATION_INTERFACE),
                    "Activate",
                    &(platform_data,),
                )
                .await
        }
        Some(page) => {
            connection
                .call_method(
                    Some(APP_NAME),
                    APPLICATION_PATH,
                    Some(APPLICATION_INTERFACE),
                    "ActivateAction",
                    &(OPEN_PAGE, vec![Value::from(page.0.as_str())], platform_data),
                )
                .await
        }
    }
    .map(|_| ())
}
