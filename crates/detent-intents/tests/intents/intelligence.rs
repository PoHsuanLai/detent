//! Settings' Intelligence action: it only shows the page. Every `ai.*` row stays the person's:
//! no action sets one and an agent's `settings.key.set` of one is refused.

use crate::support::{World, key, text};
use docket_core::AppRefusal;

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_cannot_turn_local_only_off_or_choose_a_model() {
    let world = World::start("ai-rows-never-set", &[]).await;
    for path in ["ai.local_only", "ai.floor.mail"] {
        let refusal = world
            .perform(
                "settings.key.set",
                &[("key", key(path)), ("value", text("off"))],
            )
            .await
            .expect_err("refused");
        let AppRefusal::Failed(why) = refusal else {
            panic!("{path}: {refusal:?}");
        };
        assert!(
            why.0.contains("not a setting an assistant may change"),
            "{path}: {}",
            why.0
        );
    }
    assert_eq!(world.window.told(), [], "nothing was written or shown");
}
