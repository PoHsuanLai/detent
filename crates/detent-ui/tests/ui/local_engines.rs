//! The note under "Models on this computer" when the assistant has not found llama.cpp, over a
//! fake inferd that lists the engines and a fake service manager that writes down each restart,
//! both on a private bus: the line, the command for the distro, the releases page for a release
//! with no package, nothing when it is found or the assistant is not running, and the button that
//! asks first and restarts once.

use crate::support::Sandbox;
use crate::support::bus::PrivateBus;
use crate::support::drive::{click, scroll_into_view};
use crate::support::engines::{Fake, serve};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Harness;
use std::time::Duration;

const NOTE: &str = "[data-engine-note]";
const COMMAND: &str = "[data-engine-note] .ds-command-line-text";
const START: &str = ".engine-note-start .ds-button";
const ASKED: &str = ".ds-alert";
/// What the note says over the install command.
const SETUP_LINE: &str = "To run models on this computer, install llama.cpp.";

/// What the note says when there is no package to install.
const RELEASES_LINE: &str = "Get llama.cpp's server from its releases page";

/// What the button that makes the assistant look again says.
const START_USING: &str = "Start using it";

const RESTART: &str = ".ds-alert .ds-button[data-answers=\"return\"]";
const CANCEL: &str = ".ds-alert .ds-button[data-answers=\"escape\"]";
const RESTART_CALL: &str = "RestartUnit inferd.service replace";

const FEDORA: &str = "ID=fedora\nVERSION_ID=44\n";
const UBUNTU_24: &str = "ID=ubuntu\nID_LIKE=debian\nVERSION_ID=\"24.04\"\n";

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fakes answer on real threads; a loaded machine running every test at once can take
    // several seconds, so the budget is generous (15 s) and costs nothing when it settles.
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

/// Lets whatever the window would still send reach the fakes.
fn let_it_pass(harness: &mut Harness) {
    for _ in 0..20 {
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
}

struct World {
    fake: Fake,
    harness: Harness,
    _rt: tokio::runtime::Runtime,
    _bus: PrivateBus,
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("a runtime for the fakes")
}

/// The Intelligence page on a computer of the distro `os_release` describes, with inferd (llama.cpp's
/// program there or not) and the service manager on the bus.
fn world(name: &str, os_release: &str, found: bool) -> World {
    let rt = runtime();
    let bus = PrivateBus::start(name);
    let sandbox = Sandbox::new(name).on_bus(&bus.address);
    sandbox.write_os_release(os_release);
    let fake = rt.block_on(serve(&bus, found));
    let harness = sandbox.harness(intelligence());
    World {
        fake,
        harness,
        _rt: rt,
        _bus: bus,
    }
}

#[test]
fn a_missing_llama_cpp_shows_the_line_the_command_and_the_button_and_nothing_about_the_rest() {
    let mut w = world("engines-missing", FEDORA, false);
    settle(&mut w.harness, "the note", |h| h.count(NOTE) == 1);
    let note = w.harness.text_of(NOTE).unwrap_or_default();
    assert!(note.contains(SETUP_LINE), "{note}");
    assert_eq!(
        w.harness.text_of(COMMAND).as_deref().map(str::trim),
        Some("sudo dnf install llama-cpp")
    );
    assert_eq!(w.harness.count(START), 1);
    assert!(
        w.harness
            .text_of(START)
            .unwrap_or_default()
            .contains(START_USING),
        "the button says what it does"
    );
    for quiet in ["vLLM", "Kokoro", "Speech"] {
        assert!(!note.contains(quiet), "{quiet} stays quiet: {note}");
    }
    assert!(w.fake.restarts().is_empty(), "looking restarts nothing");
}

#[test]
fn a_release_with_no_package_names_the_releases_page_and_offers_no_command() {
    let mut w = world("engines-releases-page", UBUNTU_24, false);
    settle(&mut w.harness, "the note", |h| h.count(NOTE) == 1);
    let note = w.harness.text_of(NOTE).unwrap_or_default();
    assert!(note.contains(RELEASES_LINE), "{note}");
    assert_eq!(w.harness.count(COMMAND), 0, "no command to copy");
    assert_eq!(w.harness.count(START), 1, "the restart is still offered");
}

#[test]
fn a_found_llama_cpp_and_an_assistant_that_is_not_running_show_no_note() {
    let mut found = world("engines-found", FEDORA, true);
    settle(&mut found.harness, "the look", |_| found.fake.looks() >= 1);
    let_it_pass(&mut found.harness);
    assert_eq!(found.harness.count(NOTE), 0, "found");

    let (rt, bus) = (runtime(), PrivateBus::start("engines-absent"));
    let sandbox = Sandbox::new("engines-absent").on_bus(&bus.address);
    sandbox.write_os_release(FEDORA);
    let mut away = sandbox.harness(intelligence());
    let_it_pass(&mut away);
    assert_eq!(away.count(NOTE), 0, "not running");
    drop(rt);
}

#[test]
fn start_using_it_asks_first_and_cancel_restarts_nothing() {
    let mut w = world("engines-cancel", FEDORA, false);
    settle(&mut w.harness, "the note", |h| h.count(NOTE) == 1);
    scroll_into_view(&mut w.harness, START);
    click(&mut w.harness, START);
    settle(&mut w.harness, "the question", |h| h.count(ASKED) == 1);
    let words = w.harness.text_of(ASKED).unwrap_or_default();
    assert!(words.contains("conversation that is open stops"), "{words}");
    for developer in ["systemd", "D-Bus", "inferd"] {
        assert!(!words.contains(developer), "{developer}: {words}");
    }
    assert!(w.fake.restarts().is_empty(), "asking restarts nothing");
    click(&mut w.harness, CANCEL);
    settle(&mut w.harness, "the question to go", |h| {
        h.count(ASKED) == 0
    });
    let_it_pass(&mut w.harness);
    assert!(w.fake.restarts().is_empty(), "Cancel restarts nothing");
    assert_eq!(w.harness.count(NOTE), 1, "the note stays");
}

#[test]
fn restart_restarts_the_assistant_once_and_the_page_looks_again() {
    let mut w = world("engines-restart", FEDORA, false);
    settle(&mut w.harness, "the note", |h| h.count(NOTE) == 1);
    let looks = w.fake.looks();
    scroll_into_view(&mut w.harness, START);
    click(&mut w.harness, START);
    settle(&mut w.harness, "the question", |h| h.count(ASKED) == 1);
    click(&mut w.harness, RESTART);
    settle(&mut w.harness, "the restart", |_| {
        !w.fake.restarts().is_empty()
    });
    assert_eq!(w.fake.restarts(), [RESTART_CALL]);
    settle(&mut w.harness, "the note to go", |h| h.count(NOTE) == 0);
    assert!(w.fake.looks() > looks, "the engines were asked for again");
    let_it_pass(&mut w.harness);
    assert_eq!(w.fake.restarts(), [RESTART_CALL], "once");
}
