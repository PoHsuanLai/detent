//! The model picker's first state: with no assistant to look at, it is unavailable.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{Absence, AgentWord, PickerInput, PickerState};

fn plain(_: AgentWord<'_>) -> String {
    String::new()
}

#[tokio::test]
async fn a_picker_with_no_assistant_is_unavailable_and_says_why() {
    let client = builder().build();
    let picker = client.picker(plain);

    picker
        .send(PickerInput::Found(Err(Absence::NotRunning)))
        .await;

    assert!(matches!(
        picker.live().state(),
        PickerState::Unavailable(Absence::NotRunning)
    ));
}
