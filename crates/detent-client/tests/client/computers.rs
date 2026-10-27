//! The Your computers flow: Add asks Tailscale who can be added and offers them.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{AddList, ComputersFound, ComputersInput, ComputersState, ServerFault};
use detent_system::computers::FakeComputers;
use std::sync::Arc;

fn words(_: ServerFault) -> &'static str {
    "The server's form is wrong."
}

#[tokio::test]
async fn pressing_add_looks_for_the_computers_that_can_be_added() {
    let found = ComputersFound {
        computers: Vec::new(),
        guests: Vec::new(),
    };
    let link = FakeComputers::listing(found.clone());
    let client = builder().with_computers(Arc::new(link)).build();
    let computers = client.computers(words);

    computers.send(ComputersInput::Found(Ok(found))).await;
    computers.send(ComputersInput::OpenAdd).await;

    let ComputersState::Ready(ready) = computers.live().state() else {
        panic!("the lists are there");
    };
    assert_eq!(ready.adding, AddList::Choosing(Vec::new()));
}
