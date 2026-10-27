//! The page being open: the agent is registered and devices are looked for, and both stop when the
//! session is dropped.

use super::BtSession;
use super::agent::{AGENT_PATH, Agent};
use super::read::{adapter_path, managed};
use airwaves_bluez::{Adapter1Proxy, AgentManager1Proxy, Capability};
use detent_model::Absence;
use tokio::sync::mpsc;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

fn failed<E>(_: E) -> Absence {
    Absence::Failed
}

/// What stops when the session ends.
struct Hold {
    connection: Connection,
    adapter: OwnedObjectPath,
}

impl Drop for Hold {
    fn drop(&mut self) {
        let (connection, adapter) = (self.connection.clone(), self.adapter.clone());
        // A window that is closing has no runtime left to run this on; BlueZ drops the agent and
        // stops the discovery with the connection then.
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        runtime.spawn(async move { stop(&connection, &adapter).await });
    }
}

async fn stop(connection: &Connection, adapter: &OwnedObjectPath) {
    if let Ok(builder) = Adapter1Proxy::builder(connection).path(adapter.clone())
        && let Ok(proxy) = builder.build().await
    {
        let _ = proxy.stop_discovery().await;
    }
    if let Ok(manager) = AgentManager1Proxy::new(connection).await {
        let _ = manager
            .unregister_agent(&ObjectPath::from_static_str_unchecked(AGENT_PATH))
            .await;
    }
    let _ = connection
        .object_server()
        .remove::<Agent, _>(AGENT_PATH)
        .await;
}

/// Registers the agent and starts looking for devices.
pub async fn open(connection: Connection) -> Result<BtSession, Absence> {
    let adapter = adapter_path(&managed(&connection).await?).ok_or(Absence::NoDevice)?;
    let (asks, heard) = mpsc::unbounded_channel();
    connection
        .object_server()
        .at(
            AGENT_PATH,
            Agent {
                asks,
                connection: connection.clone(),
            },
        )
        .await
        .map_err(failed)?;
    // Taken once the agent is served, so a failure from here on still drops the hold below.
    let hold = Hold {
        connection: connection.clone(),
        adapter: adapter.clone(),
    };
    let manager = AgentManager1Proxy::new(&connection).await.map_err(failed)?;
    manager
        .register_agent(
            &ObjectPath::from_static_str_unchecked(AGENT_PATH),
            Capability::KeyboardDisplay.to_wire(),
        )
        .await
        .map_err(failed)?;
    // Discovery fails while the radio is off; switching it on starts it again.
    let proxy = Adapter1Proxy::builder(&connection)
        .path(adapter)
        .map_err(failed)?
        .build()
        .await
        .map_err(failed)?;
    let _ = proxy.start_discovery().await;
    Ok(BtSession::new(heard, hold))
}
