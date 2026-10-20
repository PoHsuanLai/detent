//! The page being open: the agent is registered and devices are looked for, and both stop when the
//! session is dropped.

use super::agent::{AGENT_PATH, Agent, CAPABILITY};
use super::bluez::{AdapterProxy, AgentManagerProxy};
use super::link::BtSession;
use super::read::{adapter_path, managed};
use detent_model::Missing;
use tokio::sync::mpsc;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

fn failed<E>(_: E) -> Missing {
    Missing::Failed
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
    if let Ok(builder) = AdapterProxy::builder(connection).path(adapter.clone())
        && let Ok(proxy) = builder.build().await
    {
        let _ = proxy.stop_discovery().await;
    }
    if let Ok(manager) = AgentManagerProxy::new(connection).await {
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
pub(super) async fn open(connection: Connection) -> Result<BtSession, Missing> {
    let adapter = adapter_path(&managed(&connection).await?).ok_or(Missing::NoDevice)?;
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
    let manager = AgentManagerProxy::new(&connection).await.map_err(failed)?;
    manager
        .register_agent(
            &ObjectPath::from_static_str_unchecked(AGENT_PATH),
            CAPABILITY,
        )
        .await
        .map_err(failed)?;
    // Discovery fails while the radio is off; switching it on starts it again.
    let proxy = AdapterProxy::builder(&connection)
        .path(adapter)
        .map_err(failed)?
        .build()
        .await
        .map_err(failed)?;
    let _ = proxy.start_discovery().await;
    Ok(BtSession::new(heard, hold))
}
