//! The computers over inferd, through porter's client: the places, the Tailscale candidates, the
//! guests and the changes. The engines signal (a computer answering or not) comes from its proxy.

use crate::deadline::{Deadline, answered, peer};
use crate::live::{BusSource, confirmed, connect};
use crate::ticks::{Ticker, Ticks};
use detent_model::{Absence, ComputerRow, ComputersFound};
use ds_peer::within;
use porter_client::{
    Accounts, ClientError, ComputerName, DbusTransport, GuestAnswer, NodeId, TransportError,
};
use porter_core::lending::ComputerCandidate;
use porter_dbus::names::INFERENCE_BUS;
use porter_dbus::{BusFailure, BusStream, InferenceProxy, classify};
use zbus::Connection;

/// What the person reads when a change could not be made and the service gave no sentence.
const COULD_NOT: &str = "Your computers could not be changed. Try again in a moment.";

/// What the person reads when Settings is not allowed to change them.
const NOT_ALLOWED: &str =
    "Your computers can only be changed when Settings is opened from the desktop.";

/// One change to the computers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Change {
    /// Add the Tailscale computer.
    Add(NodeId),
    /// Remove the computer.
    Remove(ComputerName),
    /// Answer a guest.
    Answer(NodeId, GuestAnswer),
    /// Forget a guest.
    Forget(NodeId),
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

/// The sentence for a refused call: the service's own words, else the general one.
fn sentence_of(error: &ClientError) -> String {
    match error {
        ClientError::Transport(TransportError::Computer { words, .. }) if !words.is_empty() => {
            words.clone()
        }
        ClientError::Transport(TransportError::Denied(_)) => NOT_ALLOWED.to_owned(),
        _ => COULD_NOT.to_owned(),
    }
}

/// A connection, only when inferd is already on the bus: asking would start it, and opening
/// Settings must not.
pub(crate) async fn inferd(bus: &BusSource) -> Result<Connection, Absence> {
    let connection = connect(bus).await?;
    confirmed(bus, &connection, INFERENCE_BUS).await?;
    Ok(connection)
}

fn accounts(connection: &Connection) -> Accounts<DbusTransport> {
    Accounts::over(DbusTransport::over(connection.clone()))
}

/// The person's computers and the guests, as inferd lists them now.
pub(super) async fn read(bus: &BusSource) -> Result<ComputersFound, Absence> {
    let connection = inferd(bus).await?;
    let accounts = accounts(&connection);
    let asked = async {
        let places = accounts.places().await.map_err(absence_of)?;
        let guests = accounts.guests().await.map_err(absence_of)?;
        Ok::<_, Absence>((places, guests))
    };
    let (places, guests) = within(Deadline::READ, peer::ASSISTANT, asked)
        .await
        .map_err(|silent| Absence::Failed(silent.to_string()))??;
    Ok(ComputersFound {
        computers: places.iter().filter_map(ComputerRow::of).collect(),
        guests,
    })
}

/// The Tailscale computers that can be added, or the sentence that says why they cannot be listed.
pub(super) async fn candidates(bus: &BusSource) -> Result<Vec<ComputerCandidate>, String> {
    let connection = inferd(bus).await.map_err(|_| COULD_NOT.to_owned())?;
    answered(
        Deadline::READ,
        peer::ASSISTANT,
        accounts(&connection).candidates(),
    )
    .await
    .ok_or_else(|| COULD_NOT.to_owned())?
    .map_err(|e| sentence_of(&e))
}

/// Makes one change; the sentence that says why not when it was refused.
pub(super) async fn change(bus: &BusSource, change: Change) -> Result<(), String> {
    let connection = inferd(bus).await.map_err(|_| COULD_NOT.to_owned())?;
    let accounts = accounts(&connection);
    let done = answered(Deadline::ACTION, peer::ASSISTANT, async {
        match change {
            Change::Add(node) => accounts.add_tailnet_computer(&node).await.map(drop),
            Change::Remove(computer) => accounts.remove_computer(&computer).await,
            Change::Answer(node, answer) => accounts.answer_guest(&node, answer).await,
            Change::Forget(node) => accounts.forget_guest(&node).await,
        }
    })
    .await
    .ok_or_else(|| COULD_NOT.to_owned())?;
    done.map_err(|e| sentence_of(&e))
}

/// The signals that say the lists changed: the guests, and the engines (a computer answering or
/// not). Each is relayed by a task that ends with the watch (dropping it aborts them), and ticks
/// that come while one waits are one tick.
pub(super) struct Watch {
    ticks: Ticks,
}

/// Tells `out` each time `stream` yields, until the stream ends or nobody listens.
async fn relay(mut stream: impl BusStream<Item = impl Send> + Unpin, out: Ticker) {
    while std::future::poll_fn(|cx| std::pin::Pin::new(&mut stream).poll_next(cx))
        .await
        .is_some()
    {
        if out.tick().is_err() {
            return;
        }
    }
}

impl Watch {
    /// Subscribes, only when inferd is on the bus.
    pub(super) async fn start(bus: &BusSource) -> Result<Watch, Absence> {
        let connection = inferd(bus).await?;
        let subscribed = async {
            let guests = accounts(&connection)
                .watch_guests()
                .await
                .map_err(absence_of)?;
            let engines = InferenceProxy::new(&connection)
                .await
                .map_err(|e| absence_of_bus(&e))?
                .receive_engines_changed()
                .await
                .map_err(|e| absence_of_bus(&e))?;
            Ok::<_, Absence>((guests, engines))
        };
        let (guests, engines) = within(Deadline::READ, peer::ASSISTANT, subscribed)
            .await
            .map_err(|silent| Absence::Failed(silent.to_string()))??;
        let (ticker, ticks) = Ticks::channel();
        let ticks = ticks
            .with_relay(tokio::spawn(relay(guests, ticker.clone())))
            .with_relay(tokio::spawn(relay(engines, ticker)));
        Ok(Watch { ticks })
    }

    /// The next time either list changed; `None` when the bus is gone.
    pub(super) async fn changed(&mut self) -> Option<()> {
        self.ticks.recv().await
    }
}
