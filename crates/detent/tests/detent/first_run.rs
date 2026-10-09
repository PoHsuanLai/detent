//! `detent --first-run` run as the binary: once the first run has happened it exits at once, with
//! no window, no bus and nothing written, whatever the environment lacks.

use detent_files::{FirstRunMark, FirstRunStanding, StateHome};
use detent_testkit::Scratch;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_detent");
const TMP: &str = env!("CARGO_TARGET_TMPDIR");

/// How long a run that should exit at once is given.
const PATIENCE: Duration = Duration::from_secs(20);

fn run(scratch: &Scratch) -> std::process::ExitStatus {
    // Nothing of the real session reaches the process: no display, no bus, scratch directories.
    let mut child = Command::new(BIN)
        .arg("--first-run")
        .env_clear()
        .env("HOME", scratch.join("home"))
        .env("XDG_STATE_HOME", scratch.join("state"))
        .env("XDG_CONFIG_HOME", scratch.join("config"))
        .env("XDG_DATA_HOME", scratch.join("data"))
        .env("XDG_DATA_DIRS", scratch.join("dirs"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("detent starts");
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("wait") {
            return status;
        }
        if started.elapsed() > PATIENCE {
            // Killed by its PID, the one this test started.
            child.kill().expect("kill");
            child.wait().expect("reap");
            panic!("--first-run did not exit at once");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn a_second_first_run_exits_at_once_and_touches_nothing() {
    let scratch = Scratch::under(TMP, "first-run-second");
    let state = StateHome(scratch.join("state"));
    let mark = FirstRunMark::under(&state);
    mark.record().expect("the first run happened");
    assert!(run(&scratch).success());
    assert_eq!(mark.standing(), FirstRunStanding::Done);
    assert!(
        !scratch.join("config").exists(),
        "nothing was written to the config home"
    );
}
