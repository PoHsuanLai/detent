//! The Local models flow: Start on a stopped model asks inferd and shows what it answers.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{Live, ModelRun, RunAction, RunInput, RunStanding};
use detent_system::running::FakeRunning;
use std::sync::Arc;

#[tokio::test]
async fn pressing_start_on_a_stopped_model_starts_it() {
    let row = ModelRun {
        model: "local/tiny".to_owned(),
        standing: RunStanding::Stopped,
        memory_mib: 0,
    };
    let link = FakeRunning::running(vec![row.clone()]);
    let client = builder().with_running(Arc::new(link.clone())).build();
    let running = client.running();

    running.send(RunInput::Found(Ok(vec![row]))).await;
    running.send(RunInput::Act("local/tiny".to_owned())).await;

    assert_eq!(link.asked(), [("local/tiny".to_owned(), RunAction::Start)]);
    assert!(matches!(running.live().state(), Live::Ready(_)));
}
