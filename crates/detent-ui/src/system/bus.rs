//! The computer's system bus. A page looks at it and never starts anything: opening Settings must
//! not wake a service the person's session did not start. The services behind these pages are
//! system services whose names the bus's own policy reserves for the system, so the caller table
//! that vouches for the desktop's own daemons has nothing to add.

use crate::deadline::{Deadline, peer};
use crate::live::BusSource;
use detent_model::Missing;
use ds_peer::within;
use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;

/// A connection to the system bus. The session source means the real system bus; a test's private
/// bus is used as it is; no bus is no service.
pub(crate) async fn system_connection(bus: &BusSource) -> Result<Connection, Missing> {
    match within(Deadline::READ, peer::DESKTOP, dial(bus)).await {
        Ok(dialled) => dialled,
        Err(_) => Err(Missing::Failed),
    }
}

async fn dial(bus: &BusSource) -> Result<Connection, Missing> {
    match bus {
        BusSource::Absent => Err(Missing::NoService),
        BusSource::Session => Connection::system().await.map_err(|_| Missing::NoService),
        BusSource::Address(address, _) => {
            let builder = zbus::connection::Builder::address(address.as_str())
                .map_err(|_| Missing::Failed)?;
            builder.build().await.map_err(|_| Missing::NoService)
        }
    }
}

/// Whether `name` has an owner on the bus now: a look, never a start.
pub(crate) async fn is_running(connection: &Connection, name: &'static str) -> bool {
    let Ok(name) = BusName::try_from(name) else {
        return false;
    };
    let Ok(bus) = DBusProxy::new(connection).await else {
        return false;
    };
    bus.name_has_owner(name).await.unwrap_or(false)
}
