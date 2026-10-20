//! The schemas docket, almanac and cua install under `$XDG_DATA_DIRS/quire/settings`: found
//! there, each key in the section it names and the file its program reads.

use detent_model::{Catalogue, PageSlug, advanced_name, discover};
use detent_testkit::{Scratch, fixture_dir};
use std::ffi::OsString;
use std::path::Path;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

/// Installs `name`'s fixture schema into its own data directory `prefix/share`, as a package
/// does, and returns that directory.
fn install(scratch: &Scratch, prefix: &str, name: &str) -> std::path::PathBuf {
    let share = scratch.join(prefix).join("share");
    let target = share.join("quire/settings");
    std::fs::create_dir_all(&target).expect("mkdir");
    let file = format!("{name}.settings.toml");
    std::fs::copy(fixture_dir().join(&file), target.join(file)).expect("copy");
    share
}

fn catalogue(test: &str) -> Catalogue {
    let scratch = Scratch::under(TMP, test);
    let shares = [
        install(&scratch, "docket", "docket"),
        install(&scratch, "almanac", "almanac"),
        install(&scratch, "cua", "cua"),
    ];
    let joined = std::env::join_paths(&shares).expect("paths");
    let dirs = ds_settings::schema::data_dirs_from(
        Some(OsString::from(scratch.join("home/share"))),
        Some(joined),
        Some(OsString::from(scratch.join("home"))),
    );
    let found = discover(&dirs);
    assert_eq!(found.warnings, []);
    let mut apps: Vec<_> = found.programs.iter().map(|p| p.app.0.as_str()).collect();
    apps.sort_unstable();
    assert_eq!(apps, ["almanac", "cua", "docket"]);
    Catalogue::build(&found.programs)
}

fn titles(sections: &[detent_model::SectionModel]) -> Vec<(&str, Vec<&str>)> {
    sections
        .iter()
        .map(|s| {
            (
                s.title.as_str(),
                s.keys.iter().map(|k| k.spec.path.0.as_str()).collect(),
            )
        })
        .collect()
}

#[test]
fn the_three_programs_are_found_in_their_own_data_directories() {
    let catalogue = catalogue("schemas-found-in-data-dirs");
    let files: Vec<String> = catalogue
        .files
        .iter()
        .map(|f| f.file.relative().display().to_string())
        .collect();
    assert_eq!(
        files,
        [
            "docket/settings.toml",
            "almanac/settings.toml",
            "cua/settings.toml"
        ]
    );
}

#[test]
fn every_row_is_on_the_intelligence_page_in_the_section_it_names() {
    let catalogue = catalogue("schemas-sections");
    let page = catalogue
        .page(&PageSlug("intelligence".to_owned()))
        .expect("the page");
    let mut basic = titles(&page.sections);
    basic.sort();
    assert_eq!(
        basic,
        [
            (
                "Assistant",
                vec![
                    "agent.strictness",
                    "agent.mcp.expose",
                    "agent.acp.agents",
                    "agent.undo.keep_h"
                ]
            ),
            (
                "Computer use",
                vec![
                    "cua.enabled",
                    "cua.apps",
                    "cua.default_mode",
                    "cua.autopause",
                    "cua.accessibility"
                ]
            ),
            (
                "Memory",
                vec!["memory.files.at_rest", "memory.consolidation.when"]
            ),
        ]
    );
    let advanced = titles(&page.advanced);
    let advanced_keys: usize = advanced.iter().map(|(_, keys)| keys.len()).sum();
    assert_eq!(advanced_keys, 17 + 7 + 11, "docket 17, almanac 7, cua 11");
    let names: Vec<&str> = advanced.iter().map(|(title, _)| *title).collect();
    for section in [
        "Assistant",
        "Limits",
        "Memory",
        "History",
        "Run limits",
        "Timing",
        "Window detail",
    ] {
        assert!(names.contains(&section), "{section}: {names:?}");
    }
}

#[test]
fn a_key_is_held_by_its_programs_file() {
    let catalogue = catalogue("schemas-files");
    for (path, file) in [
        ("agent.mcp.expose", "docket/settings.toml"),
        ("agent.budget.calls", "docket/settings.toml"),
        ("memory.consolidation.when", "almanac/settings.toml"),
        ("memory.pending_ttl_days", "almanac/settings.toml"),
        ("cua.apps", "cua/settings.toml"),
        ("cua.tree.max_nodes", "cua/settings.toml"),
    ] {
        let entry = catalogue
            .entry(&ds_settings::schema::KeyPath(path.to_owned()))
            .unwrap_or_else(|| panic!("{path}"));
        assert_eq!(entry.file.relative(), Path::new(file), "{path}");
    }
}

#[test]
fn an_advanced_row_that_repeats_a_basic_section_says_what_is_inside() {
    let catalogue = catalogue("schemas-advanced-names");
    let page = catalogue
        .page(&PageSlug("intelligence".to_owned()))
        .expect("the page");
    let mut names: Vec<String> = page
        .advanced
        .iter()
        .map(|section| advanced_name(section, &page.sections))
        .collect();
    names.sort();
    // "Assistant" holds only Code editors and "Memory" only Apply tidying; the other sections
    // have no basic twin and keep their titles.
    assert_eq!(
        names,
        [
            "Apply tidying",
            "Code editors",
            "History",
            "Limits",
            "Run limits",
            "Timing",
            "Window detail",
        ]
    );
}
