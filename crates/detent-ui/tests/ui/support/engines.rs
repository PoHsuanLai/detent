//! A fake inferd (`org.quire.Inference1`) that answers `LocalEngines`, and a fake user service
//! manager (`org.freedesktop.systemd1.Manager`) that writes down each `RestartUnit` and, as a
//! restart does, lets inferd find llama.cpp from then on. Both are on the test's private bus.

use super::bus::PrivateBus;
use porter_dbus::names::{INFERENCE_BUS, INFERENCE_PATH};
use porter_dbus::{EngineState, LocalEngine, local_engine_to_dbus};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use zbus::fdo;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, OwnedObjectPath, OwnedValue};

type Details = HashMap<String, OwnedValue>;

const SYSTEMD_BUS: &str = "org.freedesktop.systemd1";
const SYSTEMD_PATH: &str = "/org/freedesktop/systemd1";

/// What the two fakes share.
#[derive(Default)]
struct State {
    /// Whether llama.cpp's program is there.
    found: AtomicBool,
    /// How many times `LocalEngines` was asked.
    asked: AtomicUsize,
    /// Each `RestartUnit`, as `RestartUnit <unit> <mode>`.
    calls: Mutex<Vec<String>>,
}

fn row(id: &str, name: &str, found: bool) -> (String, Details) {
    local_engine_to_dbus(&LocalEngine {
        id: id.to_owned(),
        name: name.to_owned(),
        state: if found {
            EngineState::Found(PathBuf::from(format!("/usr/bin/{id}")))
        } else {
            EngineState::Missing
        },
        slots: vec!["text".to_owned()],
        version: None,
        place: None,
    })
}

struct Inference(Arc<State>);

#[zbus::interface(name = "org.quire.Inference1")]
impl Inference {
    async fn local_engines(&self) -> Vec<(String, Details)> {
        self.0.asked.fetch_add(1, Ordering::SeqCst);
        vec![
            row(
                "llama_cpp",
                "llama.cpp",
                self.0.found.load(Ordering::SeqCst),
            ),
            row("vllm", "vLLM", false),
            row("speech", "Speech", false),
            row("kokoro", "Kokoro", false),
        ]
    }
}

struct Manager(Arc<State>);

#[zbus::interface(name = "org.freedesktop.systemd1.Manager")]
impl Manager {
    async fn restart_unit(
        &self,
        name: &str,
        mode: &str,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<OwnedObjectPath> {
        self.0
            .calls
            .lock()
            .expect("calls")
            .push(format!("RestartUnit {name} {mode}"));
        // The program is there from the restart on: the next look finds it.
        self.0.found.store(true, Ordering::SeqCst);
        let job = ObjectPath::try_from("/org/freedesktop/systemd1/job/2").expect("a path");
        Self::job_removed(&emitter, 2, job.clone(), name, "done")
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        Ok(job.into())
    }

    #[zbus(signal)]
    async fn job_removed(
        emitter: &SignalEmitter<'_>,
        id: u32,
        job: ObjectPath<'_>,
        unit: &str,
        result: &str,
    ) -> zbus::Result<()>;
}

/// The two fakes, kept alive while the test runs.
pub struct Fake {
    state: Arc<State>,
    _inferd: zbus::Connection,
    _systemd: zbus::Connection,
}

impl Fake {
    /// Every restart asked for, in order.
    pub fn restarts(&self) -> Vec<String> {
        self.state.calls.lock().expect("calls").clone()
    }

    /// How many times the engines were asked for.
    pub fn looks(&self) -> usize {
        self.state.asked.load(Ordering::SeqCst)
    }
}

/// Serves inferd, with llama.cpp's program there or not, and the service manager.
pub async fn serve(bus: &PrivateBus, found: bool) -> Fake {
    let state = Arc::new(State::default());
    state.found.store(found, Ordering::SeqCst);
    let inferd = bus.connect().await;
    inferd
        .object_server()
        .at(INFERENCE_PATH, Inference(state.clone()))
        .await
        .expect("inferd served");
    inferd
        .request_name(INFERENCE_BUS)
        .await
        .expect("inferd's name");
    let systemd = bus.connect().await;
    systemd
        .object_server()
        .at(SYSTEMD_PATH, Manager(state.clone()))
        .await
        .expect("the manager served");
    systemd
        .request_name(SYSTEMD_BUS)
        .await
        .expect("the manager's name");
    Fake {
        state,
        _inferd: inferd,
        _systemd: systemd,
    }
}
