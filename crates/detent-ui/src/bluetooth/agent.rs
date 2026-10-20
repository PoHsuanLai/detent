//! The agent this window serves to BlueZ while the page is open: when a device pairs, BlueZ asks
//! the agent of the program that started the pairing, and the agent asks the person.

use super::bluez::DeviceProxy;
use super::link::{PairingReply, PairingRequest};
use detent_model::{BtAddress, PairingAnswer, PairingAsk, PairingCode, PairingQuestion};
use tokio::sync::mpsc;
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

/// Where the agent is served.
pub(super) const AGENT_PATH: &str = "/org/quire/settings/bluetooth/agent";

/// What the agent tells BlueZ it can do: show a number and take a yes or a no.
pub(super) const CAPABILITY: &str = "DisplayYesNo";

/// How the agent refuses: BlueZ reads this error as the person saying no.
#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.bluez.Error")]
pub(super) enum AgentError {
    #[zbus(error)]
    ZBus(zbus::Error),
    Rejected(String),
}

fn rejected() -> AgentError {
    AgentError::Rejected("The person said no.".to_owned())
}

/// Forwards what devices ask to the page.
pub(super) struct Agent {
    pub(super) asks: mpsc::UnboundedSender<PairingRequest>,
    pub(super) connection: Connection,
}

impl Agent {
    /// The question `ask` puts for the device at `device`.
    async fn question(&self, device: &ObjectPath<'_>, ask: PairingAsk) -> PairingQuestion {
        let proxy = match DeviceProxy::builder(&self.connection)
            .path(OwnedObjectPath::from(device.clone()))
        {
            Ok(builder) => builder.build().await.ok(),
            Err(_) => None,
        };
        let (address, name) = match proxy {
            Some(proxy) => (
                proxy.address().await.unwrap_or_default(),
                proxy.alias().await.unwrap_or_default(),
            ),
            None => (String::new(), String::new()),
        };
        let name = if name.is_empty() {
            address.clone()
        } else {
            name
        };
        PairingQuestion {
            address: BtAddress(address),
            name,
            ask,
        }
    }

    /// Puts `ask` to the person and waits for the answer.
    async fn put(&self, device: &ObjectPath<'_>, ask: PairingAsk) -> Result<(), AgentError> {
        let question = self.question(device, ask).await;
        let (reply, answered) = PairingReply::channel();
        self.asks
            .send(PairingRequest { question, reply })
            .map_err(|_| rejected())?;
        match answered.await {
            Ok(PairingAnswer::Yes) => Ok(()),
            Ok(PairingAnswer::No) | Err(_) => Err(rejected()),
        }
    }
}

#[zbus::interface(name = "org.bluez.Agent1")]
impl Agent {
    async fn release(&self) {}

    async fn cancel(&self) {}

    async fn request_authorization(&self, device: ObjectPath<'_>) -> Result<(), AgentError> {
        self.put(&device, PairingAsk::Allow).await
    }

    async fn request_confirmation(
        &self,
        device: ObjectPath<'_>,
        passkey: u32,
    ) -> Result<(), AgentError> {
        self.put(&device, PairingAsk::Compare(PairingCode(passkey)))
            .await
    }

    /// A number to type on the device. BlueZ repeats the call as digits are typed; only the first
    /// opens the question.
    async fn display_passkey(&self, device: ObjectPath<'_>, passkey: u32, entered: u16) {
        if entered == 0 {
            let question = self
                .question(&device, PairingAsk::Show(PairingCode(passkey)))
                .await;
            let (reply, _heard) = PairingReply::channel();
            let _ = self.asks.send(PairingRequest { question, reply });
        }
    }

    async fn authorize_service(
        &self,
        _device: ObjectPath<'_>,
        _uuid: &str,
    ) -> Result<(), AgentError> {
        Ok(())
    }

    async fn request_pin_code(&self, _device: ObjectPath<'_>) -> Result<String, AgentError> {
        Err(rejected())
    }

    async fn request_passkey(&self, _device: ObjectPath<'_>) -> Result<u32, AgentError> {
        Err(rejected())
    }
}
