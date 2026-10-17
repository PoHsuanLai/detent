//! A service confirmed to be the one the desktop trusts, and called by the owner that was
//! confirmed.

use super::bus::BusSource;
use super::io::{confirmed, connect};
use detent_model::Absence;
use ds_settings::live::SenderName;
use zbus::Connection;

/// A connection, and the unique name of the owner that held a service's role in the desktop's
/// caller table when it was looked for. A call goes to `owner`, never to the well-known name: a
/// name that changes hands between the check and the call is then not called, and the call fails
/// instead of reaching whoever took it.
#[derive(Debug, Clone)]
pub(crate) struct Pinned {
    pub(crate) connection: Connection,
    pub(crate) owner: SenderName,
}

impl Pinned {
    /// The owner as a destination for a proxy builder.
    pub(crate) fn destination(&self) -> String {
        self.owner.0.clone()
    }
}

/// The connection and the owner of the service `name`, only when it is already on the bus and
/// its owner holds the service's role: `NotRunning` when nobody owns the name, `Unavailable` when
/// its owner is not the service. Nothing is started.
pub(crate) async fn pinned(bus: &BusSource, name: &str) -> Result<Pinned, Absence> {
    let connection = connect(bus).await?;
    let owner = confirmed(bus, &connection, name).await?;
    Ok(Pinned { connection, owner })
}
