//! The client's integration tests: one executable, one module per pane, each a flow over the
//! in-memory links of `detent-system` (feature `testing`). A bus is never reached: the client is
//! built over `BusSource::Absent`.

mod accounts;
mod bluetooth;
mod computers;
mod displays;
mod engines;
mod memory_consent;
mod picker;
mod running;
mod screen_sharing;
mod spaces;
mod support;
mod updates;
mod wifi;

/// Fails when a topic file in this directory is not declared above, so a new file cannot be
/// left out of the binary without anyone noticing.
#[test]
fn every_topic_file_is_declared_as_a_module() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/client");
    let main = include_str!("main.rs");
    let mut missing = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        panic!("the tests directory");
    };
    for entry in entries.flatten() {
        let path = entry.path();
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
    assert!(
        missing.is_empty(),
        "topics not declared in main.rs: {missing:?}"
    );
}
