//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).

mod agent_marks;
mod catalogue;
mod discovery;
mod engines;
mod hot_corners;
mod intelligence_schemas;
mod memory_consent;
mod sync_now;
mod update_limits;

/// Fails when a topic file in this directory is not declared above, so a new file cannot be
/// left out of the binary without anyone noticing.
#[test]
fn every_topic_file_is_declared_as_a_module() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/model");
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
