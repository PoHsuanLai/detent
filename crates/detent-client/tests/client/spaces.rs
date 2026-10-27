//! The Spaces flow: Remove asks first, then removes the Space and reads the list again.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{SpaceKey, SpaceRow, SpacesInput, SpacesState};
use detent_system::spaces::FakeSpaces;
use std::sync::Arc;

#[tokio::test]
async fn a_removed_space_is_asked_of_the_registry_and_leaves_the_list() {
    let work = SpaceRow {
        id: SpaceKey("work".to_owned()),
        name: "Work".to_owned(),
    };
    let link = FakeSpaces::holding(vec![work.clone()]);
    let client = builder().with_spaces(Arc::new(link.clone())).build();
    let spaces = client.spaces();

    spaces
        .send(SpacesInput::Found(Ok(vec![work.clone()])))
        .await;
    spaces.send(SpacesInput::AskRemove(work.id.clone())).await;
    spaces.send(SpacesInput::Confirm).await;

    let asked = link.removals();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].0, work.id);
    let SpacesState::Ready(ready) = spaces.live().state() else {
        panic!("the list is there");
    };
    assert!(ready.rows.is_empty(), "{:?}", ready.rows);
}
