//! The screen-sharing flow: Remove takes an app back and the list is read again.

use crate::support::builder;
use detent_client::Pane;
use detent_model::{
    Live, ProgramSubject, ScreenGrant, ScreenSharingInput, ShareLength, UnixSeconds,
};
use detent_system::screen_sharing::FakeScreenSharing;
use std::sync::Arc;

#[tokio::test]
async fn removing_an_app_takes_it_back_and_it_leaves_the_list() {
    let app = ProgramSubject("org.example.Call".to_owned());
    let grant = ScreenGrant {
        app: app.clone(),
        length: ShareLength::UntilRemoved,
        granted: UnixSeconds(5),
    };
    let link = FakeScreenSharing::granting(vec![grant.clone()]);
    let client = builder()
        .with_screen_sharing(Arc::new(link.clone()))
        .build();
    let sharing = client.screen_sharing();

    sharing
        .send(ScreenSharingInput::Found(Ok(vec![grant])))
        .await;
    sharing.send(ScreenSharingInput::Remove(app.clone())).await;

    assert_eq!(link.revoked(), [app]);
    let Live::Ready(ready) = sharing.live().state() else {
        panic!("the list is there");
    };
    assert!(ready.grants.is_empty());
}
