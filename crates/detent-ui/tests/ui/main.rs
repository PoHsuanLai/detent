//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).

mod account_banner;
mod account_notice;
mod accounts;
mod activation;
mod assistant_model;
mod assistants;
mod behaviour;
mod bluetooth;
mod coherence;
mod confirmed_owner;
mod displays;
mod engine_install;
mod extensions;
mod first_run;
mod holds;
mod hot_corners;
mod intelligence;
mod intelligence_programs;
mod lending;
mod local_engines;
mod local_runtimes;
mod memory_consent;
mod notices;
mod privacy;
mod screen_sharing;
mod shortcuts;
mod snapshots;
mod software_update;
mod spaces;
mod style_notes;
mod support;
mod thumbnails;
mod viewer_page;
mod wallpaper_page;
mod wifi;

/// Fails when a topic file in this directory is not declared above, so a new file cannot be
/// left out of the binary without anyone noticing.
#[test]
fn every_topic_file_is_declared_as_a_module() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui");
    let main = include_str!("main.rs");
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("the tests directory") {
        let path = entry.expect("an entry").path();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        let topic = path.is_dir() || path.extension().is_some_and(|e| e == "rs");
        if topic && stem != "main" && !main.contains(&format!("mod {stem};")) {
            missing.push(stem);
        }
    }
    assert!(missing.is_empty(), "not declared in main.rs: {missing:?}");
}
