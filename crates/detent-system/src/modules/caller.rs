//! The caller a fake module sees: the Settings app. A module's `permit` is handed the connection
//! that received the call, so the fake holds a private pair of connections to each other in this
//! process; no bus is reached and nobody else can speak on it.

use ds_settings::live::{Caller, LiveError, SenderName};
use tokio::net::UnixStream;
use tokio::sync::OnceCell;
use zbus::connection::Builder;
use zbus::{Connection, Guid};

/// The name the Settings app has on the wire the module is asked about.
const SENDER: &str = ":1.settings";

/// The Settings app as a module's `permit` is asked about it: built when first needed.
#[derive(Debug, Default)]
pub(crate) struct Settings(OnceCell<(Connection, Connection)>);

impl Settings {
    /// The caller of every call the fake makes on a module.
    pub(crate) async fn caller(&self) -> Result<Caller, LiveError> {
        let (receiving, _calling) = self.0.get_or_try_init(pair).await?;
        Ok(Caller {
            sender: SenderName(SENDER.to_owned()),
            connection: receiving.clone(),
        })
    }
}

/// Two connections to each other: the one that receives the calls and the one that makes them.
async fn pair() -> Result<(Connection, Connection), LiveError> {
    let (calling, receiving) =
        UnixStream::pair().map_err(|why| LiveError::Failed(why.to_string()))?;
    let receiving = Builder::unix_stream(receiving)
        .server(Guid::generate())
        .map_err(LiveError::ZBus)?
        .p2p()
        .build();
    let calling = Builder::unix_stream(calling).p2p().build();
    let (receiving, calling) = tokio::try_join!(receiving, calling).map_err(LiveError::ZBus)?;
    Ok((receiving, calling))
}
