//! The two calls about the engines. Looking never starts the assistant: with it not running there
//! is nothing to say, and opening Settings must not start it. Restarting is the person's word.

use crate::computers::inferd;
use crate::live::{BusSource, connect};
use detent_model::{ListedEngine, LlamaCpp, Program, Restarted, llama_cpp_of};
use porter_client::{DbusTransport, INFERD_UNIT, SessionUnits, UnitManager};
use porter_dbus::EngineState;

/// Where llama.cpp stands in the engines the assistant lists now. The assistant not running, not
/// answering, or not allowing Settings to ask leaves it unknown.
pub(super) async fn look(bus: &BusSource) -> LlamaCpp {
    let Ok(connection) = inferd(bus).await else {
        return LlamaCpp::Unknown;
    };
    let Ok(found) = DbusTransport::over(connection).list_local_engines().await else {
        return LlamaCpp::Unknown;
    };
    let listed: Vec<ListedEngine> = found
        .rows
        .into_iter()
        .map(|engine| ListedEngine {
            id: engine.id,
            program: match engine.state {
                EngineState::Found(_) => Program::Here,
                EngineState::Missing => Program::Missing,
            },
        })
        .collect();
    llama_cpp_of(&listed)
}

/// Restarts the assistant in the person's own session, and returns when it is up again.
pub(super) async fn restart(bus: &BusSource) -> Restarted {
    let Ok(connection) = connect(bus).await else {
        return Restarted::Failed;
    };
    match SessionUnits::new(connection).restart(INFERD_UNIT).await {
        Ok(()) => Restarted::Done,
        Err(_) => Restarted::Failed,
    }
}
