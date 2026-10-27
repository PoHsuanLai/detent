//! A service confirmed to be the one the desktop trusts, and called by the owner that was
//! confirmed.

use super::activate::start_one;
use super::io::{confirmed, connect};
use super::source::BusSource;
use detent_model::Absence;
use ds_settings::live::SenderName;
use porter_client::{DbusTransport, Destination};
use zbus::Connection;
use zbus::names::UniqueName;

/// A connection, and the unique name of the owner that held a service's role in the desktop's
/// caller table when it was looked for. A call goes to `owner`, never to the well-known name: a
/// name that changes hands between the check and the call is then not called, and the call fails
/// instead of reaching whoever took it.
#[derive(Debug, Clone)]
pub struct Pinned {
    pub connection: Connection,
    pub owner: SenderName,
}

impl Pinned {
    /// The owner as a destination for a proxy builder.
    pub fn destination(&self) -> String {
        self.owner.0.clone()
    }

    /// The owner as the unique name porter's client sends to. A name the bus would not have given
    /// is a service that cannot be confirmed.
    fn unique(&self) -> Result<UniqueName<'static>, Absence> {
        UniqueName::try_from(self.owner.0.clone()).map_err(|_| Absence::Unavailable)
    }

    /// Porter's carrier with every call to accountd (the sheets, the Spaces registry) sent to the
    /// owner that was confirmed.
    pub fn accounts(&self) -> Result<DbusTransport, Absence> {
        Ok(DbusTransport::over(self.connection.clone()).accounts_at(self.unique()?))
    }

    /// Porter's carrier with every call to inferd (the computers, the engines) sent to the owner
    /// that was confirmed.
    pub fn inference(&self) -> Result<DbusTransport, Absence> {
        Ok(DbusTransport::over(self.connection.clone()).inference_at(self.unique()?))
    }

    /// The confirmed owner as the destination of one of porter's per-interface clients
    /// (`AiKeys::connect_at`).
    pub fn at_owner(&self) -> Result<Destination, Absence> {
        Ok(Destination::Owner(self.unique()?))
    }
}

/// The connection and the owner of the service `name`, only when it is already on the bus and
/// its owner holds the service's role: `NotRunning` when nobody owns the name, `Unavailable` when
/// its owner is not the service. Nothing is started.
pub async fn pinned(bus: &BusSource, name: &str) -> Result<Pinned, Absence> {
    let connection = connect(bus).await?;
    let owner = confirmed(bus, &connection, name).await?;
    Ok(Pinned { connection, owner })
}

/// As [`pinned`], but a service nobody owns yet is asked to start first, for a call the person
/// just asked for (adding an account, removing a Space) or a page that is already starting it.
/// The call then goes to whoever owns the name once it is up and holds the service's role, never
/// to the name itself.
pub async fn pinned_starting(bus: &BusSource, name: &str) -> Result<Pinned, Absence> {
    let connection = connect(bus).await?;
    let owner = match confirmed(bus, &connection, name).await {
        Err(Absence::NotRunning) => {
            start_one(&connection, name).await;
            confirmed(bus, &connection, name).await?
        }
        other => other?,
    };
    Ok(Pinned { connection, owner })
}

/// inferd and its confirmed owner, only when it is already on the bus: asking would start it, and
/// opening Settings must not. Every call goes to that owner.
pub async fn inferd(bus: &BusSource) -> Result<Pinned, Absence> {
    pinned(bus, porter_dbus::names::INFERENCE_BUS).await
}
