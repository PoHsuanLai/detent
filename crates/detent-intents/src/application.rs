//! `org.freedesktop.Application` at `/org/quire/Settings`: the Desktop Entry spec's D-Bus
//! activation, so the launcher starts Settings (as its user unit), raises it, or opens it at an
//! account. A second `detent` hands over through the same `Activate`.

use crate::callers::{Refusals, admit, settings_owner};
use crate::serve::BUS_PEER;
use detent_callers::{BusPeers, Refused, desktop_peers};
use detent_model::{AccountTarget, Notice, Outbox, PageSlug};
use ds_peer::{Deadline, within};
use std::collections::HashMap;
use std::sync::Arc;
use zbus::fdo;
use zbus::message::Header;
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
    window: Outbox,
    peers: Arc<BusPeers>,
    refusals: Refusals,
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
    /// Show the window and raise it. Only the shell, another Settings and the agent tier may ask.
    async fn activate(
        &self,
        platform_data: PlatformData,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        admit(&self.peers, &self.refusals, &header).await?;
        self.window
            .send(Notice::Raise(token_of(&platform_data)))
            .map_err(|_| gone())
    }

    /// Settings opens no files.
    async fn open(&self, _uris: Vec<String>, _platform_data: PlatformData) -> fdo::Result<()> {
        Err(fdo::Error::NotSupported(
            "Settings does not open files".to_owned(),
        ))
    }

    /// `open-account` shows the Accounts page at the account named, `open-page` the page whose
    /// slug is named; an unknown action is logged and does nothing. Only the callers `Activate`
    /// admits may ask.
    async fn activate_action(
        &self,
        action_name: String,
        parameter: Vec<OwnedValue>,
        platform_data: PlatformData,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<()> {
        admit(&self.peers, &self.refusals, &header).await?;
        match action_name.as_str() {
            OPEN_ACCOUNT => self
                .window
                .send(Notice::OpenAccount(target_of(&parameter)))
                .map_err(|_| gone())?,
            OPEN_PAGE => {
                if let Some(page) = page_of(&parameter) {
                    self.window
                        .send(Notice::OpenPage(page))
                        .map_err(|_| gone())?;
                }
            }
            _ => {
                eprintln!("detent: no action {action_name:?}, ignored");
                return Ok(());
            }
        }
        self.window
            .send(Notice::Raise(token_of(&platform_data)))
            .map_err(|_| gone())
    }
}

/// Puts the activation object on `connection`, judging its callers with `peers`.
pub(crate) async fn export_application(
    connection: &zbus::Connection,
    window: Outbox,
    peers: Arc<BusPeers>,
) -> Result<(), docket_client::TransportError> {
    let application = Application {
        window,
        peers,
        refusals: Refusals::default(),
    };
    connection
        .object_server()
        .at(APPLICATION_PATH, application)
        .await
        .map(|_| ())
        .map_err(|e| docket_client::TransportError::Bus(e.to_string()))
}

/// Why a second `detent` could not hand over.
#[derive(Debug, thiserror::Error)]
pub enum HandOverFault {
    /// The program that owns the name is not Settings, so nothing was sent to it: not the page,
    /// and not the activation token.
    #[error(
        "the program that owns org.quire.Settings is not Settings ({0}), so nothing was sent to it"
    )]
    NotSettings(Refused),
    /// The bus or the running Settings did not answer as it should.
    #[error("{0}")]
    Bus(#[from] zbus::Error),
}

/// Asks the running Settings to show itself: what a second `detent` does instead of opening a
/// second window. `token` is the xdg-activation token this process was started with, if any;
/// `page` is the page the second `detent` was asked to open, if any. The owner of
/// `org.quire.Settings` must hold the Settings role first, judged by `peers`, and is then called
/// by its own unique name, so a program that takes the name meanwhile hears nothing.
pub async fn hand_over(
    connection: &zbus::Connection,
    peers: &BusPeers,
    token: Option<String>,
    page: Option<PageSlug>,
) -> Result<(), HandOverFault> {
    let asked = async {
        let owner = peers
            .owner_of(APP_NAME, &settings_owner())
            .await
            .map_err(HandOverFault::NotSettings)?;
        ask_to_show(connection, owner.as_str(), token, page)
            .await
            .map_err(HandOverFault::Bus)
    };
    within(Deadline::READ, BUS_PEER, asked)
        .await
        .map_err(|silent| HandOverFault::Bus(zbus::Error::Failure(silent.to_string())))?
}

/// [`hand_over`] on the session bus, connecting first and judging with the desktop's caller
/// table: the whole of it, connecting included, gets the read deadline, so a stuck bus socket
/// ends a second `detent` instead of hanging it.
pub async fn hand_over_session(
    token: Option<String>,
    page: Option<PageSlug>,
) -> Result<(), HandOverFault> {
    let handed = async {
        let connection = zbus::connection::Builder::session()?.build().await?;
        let peers = desktop_peers(&connection);
        hand_over(&connection, &peers, token, page).await
    };
    within(Deadline::READ, BUS_PEER, handed)
        .await
        .map_err(|silent| HandOverFault::Bus(zbus::Error::Failure(silent.to_string())))?
}

async fn ask_to_show(
    connection: &zbus::Connection,
    owner: &str,
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
                    Some(owner),
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
                    Some(owner),
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
