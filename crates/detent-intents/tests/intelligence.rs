//! Settings' Intelligence action: it only shows the page. Every `ai.*` row stays the person's:
//! no action sets one and an agent's `settings.key.set` of one is refused.

mod support;

use detent_intents::manifest;
use docket_core::AppRefusal;
use prov::Effect;
use support::{Told, World, key, text};

#[tokio::test(flavor = "multi_thread")]
async fn opening_the_intelligence_settings_tells_the_window_to_show_the_page() {
    let world = World::start("open-intelligence", &[]).await;
    world
        .perform("settings.intelligence.open", &[])
        .await
        .expect("opened");
    assert_eq!(world.window.told(), [Told::Page("intelligence".to_owned())]);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_manifest_offers_only_to_show_the_intelligence_page() {
    let manifest = manifest().expect("a manifest");
    let intelligence: Vec<_> = manifest
        .manifest()
        .actions
        .iter()
        .filter(|action| action.name.as_str().starts_with("settings.intelligence."))
        .collect();
    let names: Vec<&str> = intelligence.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(names, ["settings.intelligence.open"]);
    assert!(
        intelligence
            .iter()
            .all(|action| action.effect == Effect::Read)
    );
}

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
