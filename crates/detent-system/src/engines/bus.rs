//! The two calls about the engines. Looking never starts the assistant: with it not running there
//! is nothing to say, and opening Settings must not start it. Restarting is the person's word.

use super::EnginesLink;
use crate::bus::{BusSource, connect, inferd};
use crate::deadline::{Deadline, answered, peer};
use detent_model::{ListedEngine, LlamaCpp, Program, Restarted, Work, llama_cpp_of};
use porter_client::{INFERD_UNIT, SessionUnits, UnitManager};
use porter_dbus::inference::EngineState;

/// Where llama.cpp stands in the engines the assistant lists now. The assistant not running, not
/// answering, or not allowing Settings to ask leaves it unknown.
pub async fn look(bus: &BusSource) -> LlamaCpp {
    let Ok(running) = inferd(bus).await else {
        return LlamaCpp::Unknown;
    };
    let Ok(transport) = running.inference() else {
        return LlamaCpp::Unknown;
    };
    let Some(Ok(found)) = answered(
        Deadline::READ,
        peer::ASSISTANT,
        transport.list_local_engines(),
    )
    .await
    else {
        return LlamaCpp::Unknown;
    };
    let listed: Vec<ListedEngine> = found
        .rows
        .into_iter()
        .map(|engine| ListedEngine {
            id: engine.id,
            program: match engine.state {
                EngineState::Found(_) => Program::Here,
                EngineState::Missing | EngineState::Changed(_) => Program::Missing,
            },
        })
        .collect();
    llama_cpp_of(&listed)
}

/// Restarts the assistant in the person's own session, and returns when it is up again.
pub async fn restart(bus: &BusSource) -> Restarted {
    let Ok(connection) = connect(bus).await else {
        return Restarted::Failed;
    };
    let units = SessionUnits::new(connection);
    match answered(Deadline::ACTION, peer::SERVICES, units.restart(INFERD_UNIT)).await {
        Some(Ok(())) => Restarted::Done,
        Some(Err(_)) | None => Restarted::Failed,
    }
}

/// The assistant on the bus it is built over.
#[derive(Debug)]
pub struct OnBus(pub BusSource);

impl EnginesLink for OnBus {
    fn look(&self) -> Work<'_, LlamaCpp> {
        Box::pin(look(&self.0))
    }

    fn restart(&self) -> Work<'_, Restarted> {
        Box::pin(restart(&self.0))
    }
}
