//! A fake inferd (`org.quire.Inference1`) for the Local models group: `Running`, `Start` and `Stop`
//! with the `RunningChanged` signal, on a private bus. A Start can be held until the test lets it
//! finish, a call can be refused with inferd's own sentence, and every call is written down.

use super::bus::PrivateBus;
use super::inferd::Inferd;
use porter_dbus::names::{INFERENCE_BUS, INFERENCE_PATH};
use porter_dbus::{Details, RunState, RunningModel, running_to_dbus};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

/// The memory a ready model takes, in MiB (14.2 GB).
pub const READY_MIB: u32 = 14_540;

#[derive(Default)]
struct State {
    rows: Mutex<Vec<RunningModel>>,
    calls: Mutex<Vec<String>>,
    /// The sentence the next Start or Stop is refused with.
    refuse: Mutex<Option<String>>,
    /// Whether a Start waits in `starting` until `release`.
    held: Mutex<bool>,
    release: Notify,
}

impl State {
    fn set(&self, model: &str, state: RunState, memory_mib: u32) {
        let mut rows = self.rows.lock().expect("rows");
        if let Some(row) = rows.iter_mut().find(|row| row.model == model) {
            row.state = state;
            row.memory_mib = memory_mib;
        }
    }
}

struct Inference(Arc<State>);

impl Inference {
    fn refusal(&self) -> fdo::Result<()> {
        match self.0.refuse.lock().expect("refuse").take() {
            Some(words) => Err(fdo::Error::Failed(words)),
            None => Ok(()),
        }
    }
}

#[zbus::interface(name = "org.quire.Inference1")]
impl Inference {
    async fn running(&self) -> Vec<(String, Details)> {
        let rows = self.0.rows.lock().expect("rows");
        rows.iter().map(running_to_dbus).collect()
    }

    async fn start(
        &self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        model: String,
    ) -> fdo::Result<()> {
        self.0
            .calls
            .lock()
            .expect("calls")
            .push(format!("start {model}"));
        self.refusal()?;
        self.0.set(&model, RunState::Starting, 0);
        let _ = Self::running_changed(&emitter).await;
        let held = *self.0.held.lock().expect("held");
        if held {
            self.0.release.notified().await;
        }
        self.0.set(&model, RunState::Ready, READY_MIB);
        let _ = Self::running_changed(&emitter).await;
        Ok(())
    }

    async fn stop(
        &self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        model: String,
    ) -> fdo::Result<()> {
        self.0
            .calls
            .lock()
            .expect("calls")
            .push(format!("stop {model}"));
        self.refusal()?;
        self.0.set(&model, RunState::Stopped, 0);
        let _ = Self::running_changed(&emitter).await;
        Ok(())
    }

    #[zbus(signal)]
    async fn running_changed(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

/// The fake, kept alive while the test runs.
pub struct Fake {
    state: Arc<State>,
    connection: zbus::Connection,
}

impl Fake {
    /// Every Start and Stop that arrived, as `start <model>` and `stop <model>`.
    pub fn calls(&self) -> Vec<String> {
        self.state.calls.lock().expect("calls").clone()
    }

    /// The next Start or Stop is refused with `words`.
    pub fn refuse_next(&self, words: &str) {
        *self.state.refuse.lock().expect("refuse") = Some(words.to_owned());
    }

    /// A Start waits, once `starting`, until `finish`.
    pub fn hold_starts(&self) {
        *self.state.held.lock().expect("held") = true;
    }

    /// Lets a held Start become ready.
    pub fn finish(&self) {
        *self.state.held.lock().expect("held") = false;
        self.state.release.notify_one();
    }

    /// The model's engine fails on its own, with `reason`; inferd says so.
    pub async fn fails(&self, model: &str, reason: &str) {
        {
            let mut rows = self.state.rows.lock().expect("rows");
            if let Some(row) = rows.iter_mut().find(|row| row.model == model) {
                row.state = RunState::Failed;
                row.reason = Some(reason.to_owned());
            }
        }
        let served = self
            .connection
            .object_server()
            .interface::<_, Inference>(INFERENCE_PATH)
            .await
            .expect("inferd is served");
        Inference::running_changed(served.signal_emitter())
            .await
            .expect("the signal goes out");
    }
}

/// Serves inferd running `rows`, with the model picker's module (`module`) on the same
/// connection when there is one.
pub async fn serve(bus: &PrivateBus, rows: Vec<RunningModel>, module: Option<&Inferd>) -> Fake {
    let state = Arc::new(State {
        rows: Mutex::new(rows),
        ..State::default()
    });
    let connection = bus.connect().await;
    if let Some(module) = module {
        module.serve_module(&connection).await;
    }
    connection
        .object_server()
        .at(INFERENCE_PATH, Inference(state.clone()))
        .await
        .expect("inferd served");
    connection
        .request_name(INFERENCE_BUS)
        .await
        .expect("inferd's name");
    Fake { state, connection }
}
