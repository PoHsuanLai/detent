//! The launcher finds Settings' pages and keys in intentd's index, from the file generated
//! from the catalogue; the provider itself answers no search and opens what a hit names.

use crate::support::{World, entity};
use detent_intents::search_entries;
use detent_model::{Catalogue, Notice, OriginRule, PageSlug, discover_under};
use detent_testkit::{Scratch, seed_schemas};

const TMP: &str = env!("CARGO_TARGET_TMPDIR");
const MAGNIFICATION: &str = "dock.magnification";

/// The fields docket's `ShippedEntries::parse` takes in a row; it refuses any other.
const ROW_FIELDS: [&str; 5] = ["kind", "key", "title", "subtitle", "keywords"];

fn generated(test: &str) -> (Catalogue, toml::Table) {
    let scratch = Scratch::under(TMP, test);
    let schemas = seed_schemas(&scratch.join("data"));
    let found = discover_under(
        std::slice::from_ref(&schemas),
        &OriginRule::declared(std::slice::from_ref(&schemas)),
    );
    let catalogue = Catalogue::build(&found.programs);
    let table = search_entries(&catalogue).parse().expect("a TOML file");
    (catalogue, table)
}

fn rows(table: &toml::Table) -> Vec<&toml::Table> {
    table["entries"]
        .as_array()
        .expect("the entries")
        .iter()
        .map(|row| row.as_table().expect("a table"))
        .collect()
}

#[test]
fn the_file_has_a_page_entry_per_page_and_a_key_entry_per_key() {
    let (catalogue, table) = generated("search-file");
    let rows = rows(&table);
    for row in &rows {
        assert!(
            row.keys().all(|name| ROW_FIELDS.contains(&name.as_str())),
            "{row:?}"
        );
    }
    let of = |kind: &str| -> Vec<&toml::Table> {
        rows.iter()
            .copied()
            .filter(|row| row["kind"].as_str() == Some(kind))
            .collect()
    };
    assert_eq!(of("settings.page").len(), catalogue.pages.len());
    // A path two schemas share names no key, so there can be fewer keys than rows.
    let keys = of("settings.key").len();
    assert!(keys > 0 && keys <= catalogue.entries().count(), "{keys}");
    let dock = catalogue
        .page(&PageSlug("dock".to_owned()))
        .expect("the dock page");
    let page = of("settings.page")
        .into_iter()
        .find(|row| row["key"].as_str() == Some("dock"))
        .expect("the dock page entry");
    assert_eq!(page["title"].as_str(), Some(dock.title.as_str()));
    let key = of("settings.key")
        .into_iter()
        .find(|row| row["key"].as_str() == Some(MAGNIFICATION))
        .expect("the magnification entry");
    let spec = &catalogue
        .entry(&ds_settings::schema::KeyPath(MAGNIFICATION.to_owned()))
        .expect("the key")
        .spec;
    assert_eq!(key["title"].as_str(), Some(spec.label.0.as_str()));
    assert_eq!(key["subtitle"].as_str(), Some(dock.title.as_str()));
}

#[test]
fn an_empty_catalogue_still_gives_a_file_docket_reads() {
    let table: toml::Table = search_entries(&Catalogue::default())
        .parse()
        .expect("a TOML file");
    assert!(rows(&table).is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_provider_answers_no_search() {
    let world = World::start("search-nothing", &[]).await;
    for words in ["magnification", "dock", "zzzzqqqq", ""] {
        assert_eq!(world.search(words).await, [], "{words:?}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_a_hit_shows_its_page_or_the_row_of_its_key() {
    let world = World::start("search-open", &[]).await;
    world
        .open("settings.page.open", entity("settings.page", "dock"))
        .await
        .expect("a page opens");
    world
        .open("settings.key.open", entity("settings.key", MAGNIFICATION))
        .await
        .expect("a key opens");
    let told = world.window.told();
    assert_eq!(told[0], Notice::OpenPage(PageSlug("dock".to_owned())));
    assert!(matches!(told[1], Notice::OpenTarget(_)), "{told:?}");
}
