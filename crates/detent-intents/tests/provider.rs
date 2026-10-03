//! Settings' three actions, called over a private bus the way intentd calls an app.

mod support;

use docket_core::{AppRefusal, Preview, UndoFault, Undoable, Value};
use prov::Integrity;
use support::{Told, World, entity, key, text};

const MAGNIFICATION: &str = "dock.magnification";
const TILE_SIZE: &str = "dock.tile_size_px";
const HANDS_OFF: &str = "dock.autohide";
const SILL: &str = "sill/settings.toml";

async fn world(test: &str) -> World {
    World::start(test, &[MAGNIFICATION, TILE_SIZE]).await
}

fn failed(refusal: AppRefusal) -> String {
    match refusal {
        AppRefusal::Failed(why) => why.0,
        other => panic!("expected a refusal with words, got {other:?}"),
    }
}

fn page(slug: &str) -> Value {
    Value::Entity(entity("settings.page", slug))
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_a_page_tells_the_window() {
    let world = world("open-a-page").await;
    let out = world
        .perform("settings.page.open", &[("page", page("dock"))])
        .await
        .expect("opened");
    assert_eq!(out.undo, Undoable::No);
    assert_eq!(world.window.told(), [Told::Page("dock".to_owned())]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_page_no_schema_makes_is_not_found() {
    let world = world("open-a-missing-page").await;
    let refusal = world
        .perform("settings.page.open", &[("page", page("nowhere"))])
        .await
        .expect_err("refused");
    assert_eq!(
        refusal,
        AppRefusal::NotFound(entity("settings.page", "nowhere"))
    );
    assert_eq!(world.window.told(), []);
}

#[tokio::test(flavor = "multi_thread")]
async fn reading_a_key_answers_the_file_then_the_default_and_trusts_neither() {
    let world = world("read-a-key").await;
    let read = |path: &str| {
        let args = [("key", key(path))];
        let world = &world;
        let path = path.to_owned();
        async move {
            let out = world
                .perform("settings.key.read", &args)
                .await
                .unwrap_or_else(|e| panic!("{path}: {e:?}"));
            out.value.expect("a value")
        }
    };
    let default = read(MAGNIFICATION).await;
    assert_eq!(default.value, text("on"), "no file: the key's default");
    assert_eq!(default.label.integrity, Integrity::Untrusted);

    world.write(SILL, "[dock]\nmagnification = \"off\"\ntile_size_px = 60\n");
    assert_eq!(read(MAGNIFICATION).await.value, text("off"));
    assert_eq!(read(TILE_SIZE).await.value, text("60"));
}

#[tokio::test(flavor = "multi_thread")]
async fn any_key_can_be_read_even_one_an_agent_may_not_set() {
    let world = world("read-hands-off").await;
    world.write(SILL, "[dock]\nautohide = \"on\"\n");
    let out = world
        .perform("settings.key.read", &[("key", key(HANDS_OFF))])
        .await
        .expect("read");
    assert_eq!(out.value.expect("a value").value, text("on"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_key_nobody_describes_is_not_found() {
    let world = world("read-unknown").await;
    let refusal = world
        .perform("settings.key.read", &[("key", key("no.such_key"))])
        .await
        .expect_err("refused");
    assert_eq!(
        refusal,
        AppRefusal::NotFound(entity("settings.key", "no.such_key"))
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_garbled_file_is_not_read_as_defaults() {
    let world = world("read-garbled").await;
    world.write(SILL, "[dock\nmagnification = ");
    let refusal = world
        .perform("settings.key.read", &[("key", key(MAGNIFICATION))])
        .await
        .expect_err("refused");
    assert!(failed(refusal).contains("cannot be read"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_key_the_schema_does_not_mark_is_not_set() {
    let world = world("set-hands-off").await;
    let refusal = world
        .perform(
            "settings.key.set",
            &[("key", key(HANDS_OFF)), ("value", text("on"))],
        )
        .await
        .expect_err("refused");
    assert!(failed(refusal).contains("not marked agent-settable"));
    assert_eq!(world.read(SILL), None, "nothing was written");
    assert_eq!(world.window.told(), []);
}

#[tokio::test(flavor = "multi_thread")]
async fn setting_a_key_writes_the_file_tells_the_window_and_hands_back_an_undo() {
    let world = world("set-a-key").await;
    let before = "# mine\n[dock]\nautohide = \"off\" # kept\n[mystery]\nx = 1\n";
    world.write(SILL, before);
    let out = world
        .perform(
            "settings.key.set",
            &[("key", key(MAGNIFICATION)), ("value", text("off"))],
        )
        .await
        .expect("set");
    assert!(matches!(out.undo, Undoable::Yes(_)), "{:?}", out.undo);
    let file = world.read(SILL).expect("written");
    assert!(file.contains("magnification = \"off\""), "{file}");
    assert!(
        file.contains("autohide = \"off\" # kept"),
        "kept as it was: {file}"
    );
    assert!(file.contains("[mystery]\nx = 1"), "{file}");
    assert_eq!(world.window.told(), [Told::Files]);

    let read = world
        .perform("settings.key.read", &[("key", key(MAGNIFICATION))])
        .await
        .expect("read");
    assert_eq!(read.value.expect("a value").value, text("off"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_value_the_control_cannot_hold_is_refused_with_what_it_takes() {
    let world = world("set-bad-values").await;
    let cases = [
        (MAGNIFICATION, "maybe", "one of: on, off"),
        (TILE_SIZE, "huge", "takes a number"),
        (TILE_SIZE, "500", "from 32 to 80"),
        (TILE_SIZE, "4.5", "takes a number"),
    ];
    for (path, value, says) in cases {
        let refusal = world
            .perform(
                "settings.key.set",
                &[("key", key(path)), ("value", text(value))],
            )
            .await
            .expect_err("refused");
        let why = failed(refusal);
        assert!(why.contains(says), "{path} = {value}: {why}");
    }
    assert_eq!(world.read(SILL), None, "no refusal wrote anything");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_whole_number_argument_sets_a_number_key() {
    let world = world("set-an-integer").await;
    world
        .perform(
            "settings.key.set",
            &[("key", key(TILE_SIZE)), ("value", Value::Integer(64))],
        )
        .await
        .expect("set");
    let file = world.read(SILL).expect("written");
    assert!(file.contains("tile_size_px = 64"), "{file}");
}

#[tokio::test(flavor = "multi_thread")]
async fn undo_puts_the_old_value_back_and_only_once() {
    let world = world("undo-restores").await;
    world.write(SILL, "[dock]\ntile_size_px = 60\n");
    let out = world
        .perform(
            "settings.key.set",
            &[("key", key(TILE_SIZE)), ("value", text("40"))],
        )
        .await
        .expect("set");
    let Undoable::Yes(token) = out.undo else {
        panic!("a setting is undoable")
    };
    assert!(
        world
            .read(SILL)
            .expect("file")
            .contains("tile_size_px = 40")
    );

    world.undo(&token).await.expect("undone");
    assert!(
        world
            .read(SILL)
            .expect("file")
            .contains("tile_size_px = 60")
    );
    assert_eq!(world.window.told(), [Told::Files, Told::Files]);
    assert_eq!(world.undo(&token).await, Err(UndoFault::Gone));
}

#[tokio::test(flavor = "multi_thread")]
async fn undoing_a_key_the_file_never_had_writes_its_default() {
    let world = world("undo-default").await;
    let out = world
        .perform(
            "settings.key.set",
            &[("key", key(MAGNIFICATION)), ("value", text("off"))],
        )
        .await
        .expect("set");
    let Undoable::Yes(token) = out.undo else {
        panic!("a setting is undoable")
    };
    world.undo(&token).await.expect("undone");
    let read = world
        .perform("settings.key.read", &[("key", key(MAGNIFICATION))])
        .await
        .expect("read");
    assert_eq!(
        read.value.expect("a value").value,
        text("on"),
        "the old value is back"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn undo_does_not_overwrite_a_change_made_since() {
    let world = world("undo-conflict").await;
    let out = world
        .perform(
            "settings.key.set",
            &[("key", key(MAGNIFICATION)), ("value", text("off"))],
        )
        .await
        .expect("set");
    let Undoable::Yes(token) = out.undo else {
        panic!("a setting is undoable")
    };
    world.write(SILL, "[dock]\nmagnification = \"on\"\ntile_size_px = 33\n");
    assert_eq!(world.undo(&token).await, Err(UndoFault::Conflict));
    let file = world.read(SILL).expect("file");
    assert!(
        file.contains("tile_size_px = 33"),
        "the person's edit stays: {file}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_token_settings_never_minted_is_gone() {
    let world = world("undo-stranger").await;
    let stranger = docket_core::UndoToken::parse("mail-undo-7").expect("a token");
    assert_eq!(world.undo(&stranger).await, Err(UndoFault::Gone));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_dry_run_shows_the_change_and_makes_none() {
    let world = world("dry-run").await;
    world.write(SILL, "[dock]\ntile_size_px = 60\n");
    let args = [("key", key(TILE_SIZE)), ("value", text("40"))];
    let Preview::TextChange { before, after } = world
        .dry_run("settings.key.set", &args)
        .await
        .expect("a preview")
    else {
        panic!("a change shows as before and after")
    };
    assert_eq!((before.value.as_str(), after.value.as_str()), ("60", "40"));
    assert_eq!(
        after.label.integrity,
        Integrity::Untrusted,
        "the new value keeps the label the caller gave it"
    );
    assert!(
        world
            .read(SILL)
            .expect("file")
            .contains("tile_size_px = 60")
    );
    assert_eq!(world.window.told(), []);

    let refusal = world
        .dry_run(
            "settings.key.set",
            &[("key", key(HANDS_OFF)), ("value", text("on"))],
        )
        .await
        .expect_err("a refused change has no preview either");
    assert!(failed(refusal).contains("not marked agent-settable"));
}

#[tokio::test(flavor = "multi_thread")]
async fn suggestions_offer_pages_every_key_and_only_the_settable_ones_to_set() {
    let world = world("suggest").await;
    let keys = |things: Vec<docket_core::EntityRef>| -> Vec<String> {
        things
            .iter()
            .map(|t| t.id.key.as_str().to_owned())
            .collect()
    };
    let pages = keys(world.suggest("settings.page.open", "page", "dock").await);
    assert_eq!(pages, ["dock"]);
    let settable = keys(world.suggest("settings.key.set", "key", "dock.").await);
    assert_eq!(settable, [MAGNIFICATION, TILE_SIZE]);
    let readable = keys(world.suggest("settings.key.read", "key", "dock.").await);
    assert!(readable.contains(&HANDS_OFF.to_owned()), "{readable:?}");
    assert!(readable.len() > settable.len());
    assert_eq!(world.suggest("settings.key.set", "value", "").await, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_search_finds_keys_by_the_schemas_own_words() {
    let world = world("search").await;
    let hits = world.search("magnification").await;
    let found: Vec<&str> = hits.iter().map(|h| h.entity.id.key.as_str()).collect();
    assert!(found.contains(&MAGNIFICATION), "{found:?}");
    assert_eq!(world.search("zzzzqqqq").await, []);
}

#[tokio::test(flavor = "multi_thread")]
async fn previews_describe_a_key_and_list_a_page() {
    let world = world("preview").await;
    let Preview::Facts(lines) = world.preview(&entity("settings.key", MAGNIFICATION)).await else {
        panic!("a key previews as facts")
    };
    let labels: Vec<&str> = lines.iter().map(|l| l.label.as_str()).collect();
    assert_eq!(labels, ["Setting", "Help", "Value"]);
    let Preview::List(keys) = world.preview(&entity("settings.page", "dock")).await else {
        panic!("a page previews as a list")
    };
    assert!(!keys.is_empty());
    assert_eq!(
        world.preview(&entity("settings.page", "nowhere")).await,
        Preview::None
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_argument_missing_or_of_the_wrong_kind_is_refused() {
    let world = world("bad-arguments").await;
    let missing = world
        .perform("settings.key.set", &[("key", key(MAGNIFICATION))])
        .await
        .expect_err("refused");
    assert!(
        matches!(&missing, AppRefusal::NeedsParam { param, .. } if param.as_str() == "value"),
        "{missing:?}"
    );
    let wrong = world
        .perform("settings.key.read", &[("key", page("dock"))])
        .await
        .expect_err("refused");
    assert_eq!(wrong, AppRefusal::NotFound(entity("settings.page", "dock")));
    let unknown = world
        .perform("settings.key.delete", &[("key", key(MAGNIFICATION))])
        .await
        .expect_err("refused");
    assert_eq!(unknown, AppRefusal::Unsupported);
}
