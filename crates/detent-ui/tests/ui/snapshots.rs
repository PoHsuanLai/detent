//! Pictures of every page, light and dark, built from the real schemas. They are written to `$DETENT_SHOTS` (else the target directory's `tmp/shots`).

use crate::support::{Sandbox, TMP, VIEW};
use detent_model::PageSlug;
use detent_system::person::FakePerson;
use detent_ui::Opening;
use ds::prelude::Scheme;
use ds_harness::snapshot_with;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

fn shots_dir() -> PathBuf {
    std::env::var_os("DETENT_SHOTS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(TMP).join("shots"))
}

/// The page's picture in `scheme`, saved as `<page>-<scheme>.png`; returns how many distinct
/// colours it holds.
fn shoot(page: &str, scheme: Scheme, name: &str) -> usize {
    let sandbox =
        Sandbox::new(name).with_person(std::sync::Arc::new(FakePerson::named("Ada Lovelace")));
    let opening = Opening {
        page: Some(PageSlug(page.to_owned())),
        query: String::new(),
    };
    let config = sandbox.config(opening, scheme);
    let frames =
        snapshot_with(detent_ui::App, config, &[Duration::from_millis(500)]).expect("a picture");
    let image = frames.into_iter().next().expect("one frame");
    assert_eq!((image.width(), image.height()), (VIEW.width, VIEW.height));
    let dir = shots_dir();
    std::fs::create_dir_all(&dir).expect("shots dir");
    let file = format!(
        "{}-{}.png",
        page.replace(':', "-"),
        if scheme == Scheme::Dark {
            "dark"
        } else {
            "light"
        }
    );
    image.save(dir.join(file)).expect("save png");
    image.pixels().map(|p| p.0).collect::<HashSet<_>>().len()
}

#[test]
fn each_page_is_drawn_in_light_and_dark() {
    for page in [
        "wifi",
        "bluetooth",
        "notifications",
        "appearance",
        "extensions",
        "intelligence",
        "assistant-model",
        "privacy",
        "dock",
        "displays",
        "wallpaper",
        "spaces",
        "power",
        "software-update",
        "accounts",
        "keyboard_and_shortcuts",
        "mouse_and_gestures",
        "apps",
        "app:files",
    ] {
        let name = page.replace(':', "-");
        let light = shoot(page, Scheme::Light, &format!("shot-{name}-light"));
        let dark = shoot(page, Scheme::Dark, &format!("shot-{name}-dark"));
        assert!(
            light > 50,
            "{page} light is a real picture, not a blank window ({light} colours)"
        );
        assert!(dark > 50, "{page} dark is a real picture ({dark} colours)");
    }
}
