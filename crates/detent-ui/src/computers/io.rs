//! The computers over inferd. The places go through porter's client; the Tailscale candidates,
//! the guests and the changes it does not carry yet go through porter's typed proxy.

use super::rows::{candidate, guest};
use crate::live::{BusSource, connect};
use detent_model::{
    Absence, Candidate, ComputerRow, ComputersFound, GuestAnswer, NodeKey, refusal_sentence,
};
use porter_client::{Accounts, ClientError, DbusTransport, TransportError};
use porter_dbus::{
    BusFailure, BusStream, COMPUTER_ERROR_PREFIX, GUEST_ANSWER_ALLOW, GUEST_ANSWER_DENY,
    INFERENCE_BUS, InferenceProxy, classify,
};
use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;

/// What the person reads when a change could not be made and the service gave no sentence.
const COULD_NOT: &str = "Your computers could not be changed. Try again in a moment.";

/// What the person reads when Settings is not allowed to change them.
const NOT_ALLOWED: &str =
    "Your computers can only be changed when Settings is opened from the desktop.";

/// One change to the computers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Change {
    /// Add the Tailscale computer.
    Add(NodeKey),
    /// Remove the computer, by place id.
    Remove(String),
    /// Answer a guest.
    Answer(NodeKey, GuestAnswer),
    /// Forget a guest.
    Forget(NodeKey),
}

fn absence_of(error: ClientError) -> Absence {
    match error {
        ClientError::Transport(TransportError::Unreachable | TransportError::Closed) => {
            Absence::NotRunning
        }
        ClientError::Transport(TransportError::Denied(_)) => Absence::NotPermitted,
        other => Absence::Failed(other.to_string()),
    }
}

fn absence_of_bus(error: &zbus::Error) -> Absence {
    match classify(error) {
        BusFailure::NoDaemon => Absence::NotRunning,
        BusFailure::Denied(_) => Absence::NotPermitted,
        _ => Absence::Failed(error.to_string()),
    }
}

/// The sentence for a bus error: the service's own refusal in plain words, else the general one.
fn sentence_of(error: &zbus::Error) -> String {
    match error {
        zbus::Error::MethodError(name, _, _) => name
            .as_str()
            .strip_prefix(COMPUTER_ERROR_PREFIX)
            .map_or(COULD_NOT, refusal_sentence)
            .to_owned(),
        other => match classify(other) {
            BusFailure::Denied(_) => NOT_ALLOWED.to_owned(),
            _ => COULD_NOT.to_owned(),
        },
    }
}

/// A connection, only when inferd is already on the bus: asking would start it, and opening
/// Settings must not.
async fn inferd(bus: &BusSource) -> Result<Connection, Absence> {
    let connection = connect(bus).await?;
    let name = BusName::try_from(INFERENCE_BUS).map_err(|e| Absence::Failed(e.to_string()))?;
    let running = DBusProxy::new(&connection)
        .await
        .map_err(|e| Absence::Failed(e.to_string()))?
        .name_has_owner(name)
        .await
        .map_err(|e| Absence::Failed(e.to_string()))?;
    if running {
        Ok(connection)
    } else {
        Err(Absence::NotRunning)
    }
}

async fn proxy(connection: &Connection) -> Result<InferenceProxy<'_>, Absence> {
    InferenceProxy::new(connection)
        .await
        .map_err(|e| absence_of_bus(&e))
}

/// The person's computers and the guests, as inferd lists them now.
pub(super) async fn read(bus: &BusSource) -> Result<ComputersFound, Absence> {
    let connection = inferd(bus).await?;
    let places = Accounts::over(DbusTransport::over(connection.clone()))
        .places()
        .await
        .map_err(absence_of)?;
    let guests = proxy(&connection)
        .await?
        .guests()
        .await
        .map_err(|e| absence_of_bus(&e))?;
    Ok(ComputersFound {
        computers: places.iter().filter_map(ComputerRow::of).collect(),
        guests: guests.into_iter().filter_map(guest).collect(),
    })
}

/// The Tailscale computers that can be added, or the sentence that says why they cannot be listed.
pub(super) async fn candidates(bus: &BusSource) -> Result<Vec<Candidate>, String> {
    let connection = inferd(bus).await.map_err(|_| COULD_NOT.to_owned())?;
    let rows = proxy(&connection)
        .await
        .map_err(|_| COULD_NOT.to_owned())?
        .candidates()
        .await
        .map_err(|e| sentence_of(&e))?;
    Ok(rows.into_iter().filter_map(candidate).collect())
}

/// Makes one change; the sentence that says why not when it was refused.
pub(super) async fn change(bus: &BusSource, change: Change) -> Result<(), String> {
    let connection = inferd(bus).await.map_err(|_| COULD_NOT.to_owned())?;
    let inference = proxy(&connection).await.map_err(|_| COULD_NOT.to_owned())?;
    let done = match change {
        Change::Add(node) => inference.add_tailnet_computer(&node.0).await.map(drop),
        Change::Remove(id) => inference.remove_computer(&id).await,
        Change::Answer(node, answer) => {
            let word = match answer {
                GuestAnswer::Allow => GUEST_ANSWER_ALLOW,
                GuestAnswer::Deny => GUEST_ANSWER_DENY,
            };
            inference.answer_guest(&node.0, word).await
        }
        Change::Forget(node) => inference.forget_guest(&node.0).await,
    };
    done.map_err(|e| sentence_of(&e))
}

/// The signals that say the lists changed: the guests, and the engines (a computer answering or
/// not). Each is relayed by a task that ends with the watch.
pub(super) struct Watch {
    changes: tokio::sync::mpsc::UnboundedReceiver<()>,
}

/// Tells `out` each time `stream` yields, until the stream ends or nobody listens.
async fn relay(
    mut stream: impl BusStream<Item = impl Send> + Unpin,
    out: tokio::sync::mpsc::UnboundedSender<()>,
) {
    while std::future::poll_fn(|cx| std::pin::Pin::new(&mut stream).poll_next(cx))
        .await
        .is_some()
    {
        if out.send(()).is_err() {
            return;
        }
    }
}

impl Watch {
    /// Subscribes, only when inferd is on the bus.
    pub(super) async fn start(bus: &BusSource) -> Result<Watch, Absence> {
        let connection = inferd(bus).await?;
        let inference = proxy(&connection).await?;
        let guests = inference
            .receive_guests_changed()
            .await
            .map_err(|e| absence_of_bus(&e))?;
        let engines = inference
            .receive_engines_changed()
            .await
            .map_err(|e| absence_of_bus(&e))?;
        let (out, changes) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(relay(guests, out.clone()));
        tokio::spawn(relay(engines, out));
        Ok(Watch { changes })
    }

    /// The next time either list changed; `None` when the bus is gone.
    pub(super) async fn changed(&mut self) -> Option<()> {
        self.changes.recv().await
    }
}
