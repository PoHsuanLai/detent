//! The llama.cpp note flow: Start using it asks first, then restarts the assistant and looks again.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{EnginesInput, LlamaCpp, Restart};
use detent_system::engines::FakeEngines;
use std::sync::Arc;

#[tokio::test]
async fn an_allowed_restart_restarts_the_assistant_and_looks_again() {
    let link = FakeEngines::listing(LlamaCpp::Missing);
    let client = builder().with_engine_note(Arc::new(link.clone())).build();
    let engines = client.engines();

    engines.send(EnginesInput::Listed(LlamaCpp::Missing)).await;
    engines.send(EnginesInput::StartUsing).await;
    assert_eq!(engines.live().state().restart, Restart::Asking);

    link.lists(LlamaCpp::Found);
    engines.send(EnginesInput::Allowed).await;

    assert_eq!(link.restarts(), 1);
    assert_eq!(engines.live().state().llama_cpp, LlamaCpp::Found);
    assert_eq!(engines.live().state().restart, Restart::Idle);
}
