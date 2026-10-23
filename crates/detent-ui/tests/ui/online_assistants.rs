//! The coding assistants' Add sheet over the list on offer: a fake `docket-agents` in a scratch
//! folder stands in for the program that reaches the network, so nothing leaves the test and the
//! real folder is never touched.

use crate::support::Sandbox;
use crate::support::drive::{click, click_when_still, scroll_into_view};
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::Harness;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

const AGENTS: &str = "# mine\n[[agent]]\nprogram = \"claude-code\"\nlabel = \"Claude Code\"\nregistry = \"claude-code\"\nversion = \"1.0.0\"\n";
const INSTALLED: &str =
    "id = \"gemini\"\nversion = \"1.0.0\"\ncommand = \"bin/gemini\"\ncheck = \"declared\"\n";
const GROUP: &str = "[data-assistants=\"coding\"]";
const SHEET: &str = "[data-system=\"add-coding-assistant\"]";
const CHECKSUM: &str = "The download did not match its checksum.";
const NO_ROUTE: &str = "No route to the host.";

/// The fake: `list` offers three assistants (one already added, one installed), `install` makes
/// the folder a real install would. `{LIST}` and `{INSTALL}` are what `list` and `install` do
/// after they have said their first line.
const FAKE: &str = r#"#!/bin/sh
case "$3" in
list)
  {LIST}
  ;;
install)
  echo "installing $4 $5"
  {INSTALL}
  ;;
esac
"#;

const LISTS: &str = r#"echo '{"id":"claude-code","name":"Claude Code","description":"From Anthropic.","version":"1.0.0","installed":true}'
  echo '{"id":"codex","name":"Codex","description":"Writes code with you.","version":"2.0.0","installed":false}'
  echo '{"id":"gemini","name":"Gemini CLI","description":"From Google.","version":"1.0.0","installed":true}'"#;

const INSTALLS: &str = r#"mkdir -p "$2/installed/$4/$5"
  printf 'id = "%s"\nversion = "%s"\ncommand = "bin/x"\ncheck = "declared"\n' "$4" "$5" > "$2/installed/$4/$5/installed.toml"
  echo "installed $4 $5; pin it""#;

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The program runs on a real thread; the budget is generous and costs nothing when it settles.
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

/// A world whose program does `list` and `install` as the two scripts say.
fn world(name: &str, list: &str, install: &str) -> Sandbox {
    let sandbox = Sandbox::new(name);
    sandbox.write("docket/agents.toml", AGENTS);
    let body = FAKE.replace("{LIST}", list).replace("{INSTALL}", install);
    sandbox.write_data("bin/docket-agents", &body);
    let program: PathBuf = sandbox.data_home().0.join("bin/docket-agents");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("mode");
    sandbox.with_agents_program(program)
}

fn open_sheet(harness: &mut Harness) {
    settle(harness, "the group", |h| h.count(GROUP) == 1);
    let open = format!("{GROUP} button[aria-label=\"Add a coding assistant\"]");
    scroll_into_view(harness, &open);
    click_when_still(harness, &open);
    settle(harness, "the sheet", |h| h.count(SHEET) == 1);
}

fn sheet_text(harness: &Harness) -> String {
    harness.text_of(SHEET).unwrap_or_default()
}

#[test]
fn the_list_on_offer_is_shown_and_an_assistant_not_installed_is_downloaded_then_listed() {
    let sandbox = world("online-assistants-ok", LISTS, INSTALLS);
    let mut harness = sandbox.harness(intelligence());
    open_sheet(&mut harness);
    settle(&mut harness, "the list", |h| {
        sheet_text(h).contains("Codex")
    });
    let text = sheet_text(&harness);
    for want in [
        "Codex",
        "Writes code with you.",
        "Gemini CLI",
        "From Google.",
        "Installed",
    ] {
        assert!(text.contains(want), "{want} in {text}");
    }
    assert!(!text.contains("Claude Code"), "already added: {text}");
    for developer in ["registry", "slug", "docket-agents", "CLI tool"] {
        assert!(!text.contains(developer), "{developer} in {text}");
    }
    assert_eq!(sandbox.read("docket/agents.toml").as_deref(), Some(AGENTS));

    click(&mut harness, &format!("{SHEET} [aria-label=\"Add\"]"));
    settle(&mut harness, "the sheet closing", |h| h.count(SHEET) == 0);
    let written = sandbox.read("docket/agents.toml").expect("the file");
    for want in [
        "program = \"codex\"",
        "label = \"Codex\"",
        "version = \"2.0.0\"",
    ] {
        assert!(written.contains(want), "{want} in {written}");
    }
    assert!(written.starts_with(AGENTS), "{written}");
    assert!(
        sandbox
            .data_home()
            .0
            .join("docket/agents/installed/codex/2.0.0")
            .is_dir(),
        "downloaded"
    );
    settle(&mut harness, "the new row", |h| {
        h.text_of(GROUP).unwrap_or_default().contains("Codex")
    });
}

#[test]
fn a_download_that_fails_keeps_the_sheet_open_with_the_sentence() {
    let failing = format!("echo '{CHECKSUM}' >&2\n  exit 1");
    let sandbox = world("online-assistants-fail", LISTS, &failing);
    let mut harness = sandbox.harness(intelligence());
    open_sheet(&mut harness);
    settle(&mut harness, "the list", |h| {
        sheet_text(h).contains("Codex")
    });
    click(&mut harness, &format!("{SHEET} [aria-label=\"Add\"]"));
    settle(&mut harness, "the sentence", |h| {
        sheet_text(h).contains(CHECKSUM)
    });
    assert_eq!(harness.count(SHEET), 1, "the sheet stays");
    assert_eq!(sandbox.read("docket/agents.toml").as_deref(), Some(AGENTS));
}

#[test]
fn a_list_that_cannot_be_fetched_says_why_and_still_offers_the_installed_ones() {
    let failing = format!("echo '{NO_ROUTE}' >&2\n  exit 1");
    let sandbox = world("online-assistants-offline", &failing, INSTALLS);
    sandbox.write_data(
        "docket/agents/installed/gemini/1.0.0/installed.toml",
        INSTALLED,
    );
    let mut harness = sandbox.harness(intelligence());
    open_sheet(&mut harness);
    settle(&mut harness, "the reason", |h| {
        sheet_text(h).contains(NO_ROUTE)
    });
    let text = sheet_text(&harness);
    assert!(text.contains("can\u{2019}t be fetched"), "{text}");
    assert!(
        text.contains("Gemini"),
        "the installed one is offered: {text}"
    );

    click(&mut harness, &format!("{SHEET} [aria-label=\"Add\"]"));
    settle(&mut harness, "the sheet closing", |h| h.count(SHEET) == 0);
    let written = sandbox.read("docket/agents.toml").expect("the file");
    assert!(written.contains("program = \"gemini\""), "{written}");
}
