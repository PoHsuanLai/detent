//! The calls about the models inferd runs itself: reading them, starting and stopping one, and the
//! signal that says to read again. Looking never starts the assistant.

use crate::computers::{inferd, relay};
use crate::deadline::{Deadline, answered, peer};
use crate::live::{BusSource, absence_of_bus};
use crate::ticks::Ticks;
use detent_model::{Absence, ModelRun, RunAction, RunStanding};
use ds_peer::within;
use porter_client::TransportError;
use porter_dbus::{InferenceProxy, RunState, RunningModel};

/// What the person reads when a model could not be started or stopped and the assistant gave no
/// sentence.
const COULD_NOT: &str = "The model could not be changed. Try again in a moment.";

/// What the person reads when Settings is not allowed to start or stop models.
const NOT_ALLOWED: &str =
    "Models can only be started or stopped when Settings is opened from the desktop.";

/// A row of the list as the page reads it. A state this build does not know reads as stopped, the
/// one that offers Start and claims nothing.
fn run_of(row: &RunningModel) -> ModelRun {
    let standing = match row.state {
        RunState::Starting => RunStanding::Starting,
        RunState::Ready => RunStanding::Ready,
        RunState::Stopping => RunStanding::Stopping,
        RunState::Retrying => RunStanding::Retrying,
        RunState::Failed => RunStanding::Failed(row.reason.clone()),
        _ => RunStanding::Stopped,
    };
    ModelRun {
        model: row.model.clone(),
        standing,
        memory_mib: row.memory_mib,
    }
}

fn absence_of(error: &TransportError) -> Absence {
    match error {
        TransportError::Unreachable | TransportError::Closed => Absence::NotRunning,
        TransportError::Denied(_) => Absence::NotPermitted,
        _ => Absence::Failed,
    }
}

/// The models inferd runs itself, as it lists them now.
pub(super) async fn read(bus: &BusSource) -> Result<Vec<ModelRun>, Absence> {
    let running = inferd(bus).await?;
    let transport = running.inference()?;
    let found = answered(Deadline::READ, peer::ASSISTANT, transport.list_running())
        .await
        .ok_or(Absence::Failed)?
        .map_err(|error| absence_of(&error))?;
    Ok(found.rows.iter().map(run_of).collect())
}

/// What a refused Start or Stop says: the assistant's own words, else the general sentence.
fn words_of(error: &TransportError) -> String {
    match error {
        TransportError::NotRun(words) if !words.is_empty() => words.clone(),
        TransportError::Denied(_) => NOT_ALLOWED.to_owned(),
        _ => COULD_NOT.to_owned(),
    }
}

/// Does `action` to `model`; the words it was refused with when it was. A Start returns
/// once the model answers, which can take minutes, so it waits as long as a person's answer does;
/// a Stop is an ordinary action.
pub(super) async fn act(bus: &BusSource, model: &str, action: RunAction) -> Result<(), String> {
    let running = inferd(bus).await.map_err(|_| COULD_NOT.to_owned())?;
    let transport = running.inference().map_err(|_| COULD_NOT.to_owned())?;
    let done = match action {
        RunAction::Start => {
            answered(
                Deadline::PROMPT,
                peer::ASSISTANT,
                transport.start_model(model),
            )
            .await
        }
        RunAction::Stop => {
            answered(
                Deadline::ACTION,
                peer::ASSISTANT,
                transport.stop_model(model),
            )
            .await
        }
    };
    done.ok_or_else(|| COULD_NOT.to_owned())?
        .map_err(|error| words_of(&error))
}

/// Waits for the signal that says a model's engine changed state.
pub(super) struct Watch {
    ticks: Ticks,
}

impl Watch {
    /// Subscribes, only when inferd is on the bus.
    pub(super) async fn start(bus: &BusSource) -> Result<Watch, Absence> {
        let running = inferd(bus).await?;
        let subscribed = async {
            InferenceProxy::builder(&running.connection)
                .destination(running.destination())
                .map_err(|e| absence_of_bus(&e))?
                .build()
                .await
                .map_err(|e| absence_of_bus(&e))?
                .receive_running_changed()
                .await
                .map_err(|e| absence_of_bus(&e))
        };
        let changes = within(Deadline::READ, peer::ASSISTANT, subscribed)
            .await
            .map_err(|_| Absence::Failed)??;
        let (ticker, ticks) = Ticks::channel();
        let ticks = ticks.with_relay(tokio::spawn(relay(changes, ticker)));
        Ok(Watch { ticks })
    }

    /// The next time a model changed state; `None` when the bus is gone.
    pub(super) async fn changed(&mut self) -> Option<()> {
        self.ticks.recv().await
    }
}
