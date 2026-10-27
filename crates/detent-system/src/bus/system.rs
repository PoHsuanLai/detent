//! The computer's system bus. A page looks at it and never starts anything: opening Settings must
//! not wake a service the person's session did not start. The services behind these pages are
//! system services whose names the bus's own policy reserves for the system, so the caller table
//! that vouches for the desktop's own daemons has nothing to add.

use super::source::BusSource;
use crate::deadline::{Deadline, peer};
use detent_model::Absence;
use ds_peer::within;
use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;

/// A connection to the system bus. The session source means the real system bus; a test's private
/// bus is used as it is; no bus is no service.
pub async fn system_connection(bus: &BusSource) -> Result<Connection, Absence> {
    match within(Deadline::READ, peer::DESKTOP, dial(bus)).await {
        Ok(dialled) => dialled,
        Err(_) => Err(Absence::Failed),
    }
}

async fn dial(bus: &BusSource) -> Result<Connection, Absence> {
    match bus {
        BusSource::Absent => Err(Absence::NotRunning),
        BusSource::Session => Connection::system().await.map_err(|_| Absence::NotRunning),
        BusSource::Address(address, _) => {
            let builder = zbus::connection::Builder::address(address.as_str())
                .map_err(|_| Absence::Failed)?;
            builder.build().await.map_err(|_| Absence::NotRunning)
        }
    }
}

/// Whether `name` has an owner on the bus now: a look, never a start.
pub async fn is_running(connection: &Connection, name: &'static str) -> bool {
    let Ok(name) = BusName::try_from(name) else {
        return false;
    };
    let Ok(bus) = DBusProxy::new(connection).await else {
        return false;
    };
    bus.name_has_owner(name).await.unwrap_or(false)
}
