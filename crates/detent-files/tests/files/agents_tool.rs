//! The program that downloads coding assistants, run as a child process: a fake one in a scratch
//! folder stands in, so nothing reaches the network and the real folder is never touched.

use detent_files::{AgentsTool, DataHome, find_agents_program, installed_agents};
use detent_model::Presence;
use detent_testkit::Scratch;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Mutex;

const TMP: &str = env!("CARGO_TARGET_TMPDIR");

/// Held for the whole of each test here. A test that starts a program forks while another may be
/// writing its fake; the child then holds that file open for writing and the exec fails (ETXTBSY).
static FORKS: Mutex<()> = Mutex::new(());

/// The fake: `list` offers two assistants, `install` makes the folder a real install would.
const FAKE: &str = r#"#!/bin/sh
case "$3" in
list)
  echo '{"id":"codex","name":"Codex","description":"Writes code with you.","version":"2.0.0","installed":false}'
  echo '{"id":"gemini","name":"Gemini CLI","description":"","version":"1.0.0","installed":true}'
  echo 'not json'
  ;;
install)
  echo "installing $4 $5"
  if [ -n "$FAIL" ]; then echo "The download did not match its checksum." >&2; exit 1; fi
  mkdir -p "$2/installed/$4/$5"
  printf 'id = "%s"\nversion = "%s"\ncommand = "bin/x"\ncheck = "declared"\n' "$4" "$5" > "$2/installed/$4/$5/installed.toml"
  echo "installed $4 $5; pin it"
  ;;
esac
"#;

fn fake(scratch: &Scratch, body: &str) -> PathBuf {
    let bin = scratch.join("bin");
    std::fs::create_dir_all(&bin).expect("mkdir");
    let program = bin.join("docket-agents");
    std::fs::write(&program, body).expect("script");
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).expect("mode");
    program
}

#[test]
fn the_list_is_read_from_the_program_and_an_install_lands_in_the_folder() {
    let _forks = FORKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let scratch = Scratch::under(TMP, "agents-tool-ok");
    let data = DataHome(scratch.join("data"));
    let tool = AgentsTool::new(fake(&scratch, FAKE), &data);

    let list = tool.list().expect("listed");
    let seen: Vec<(&str, &str, Presence)> = list
        .iter()
        .map(|one| (one.id.as_str(), one.name.as_str(), one.presence))
        .collect();
    assert_eq!(
        seen,
        [
            ("codex", "Codex", Presence::Online),
            ("gemini", "Gemini CLI", Presence::Installed)
        ],
        "in the program's order, a line that is not an entry skipped"
    );
    assert_eq!(list[0].description, "Writes code with you.");
    assert_eq!(list[0].version, "2.0.0");

    let mut begun = 0;
    tool.install("codex", "2.0.0", || begun += 1)
        .expect("installed");
    assert_eq!(begun, 1, "told once, when the program says it began");
    let found: Vec<String> = installed_agents(&data).into_iter().map(|f| f.id).collect();
    assert_eq!(found, ["codex"]);
}

#[test]
fn a_program_that_fails_gives_its_sentence_and_one_that_is_missing_gives_none() {
    let _forks = FORKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let scratch = Scratch::under(TMP, "agents-tool-fail");
    let data = DataHome(scratch.join("data"));
    let failing = format!(
        "#!/bin/sh\nFAIL=1\n{}",
        FAKE.trim_start_matches("#!/bin/sh\n")
    );
    let tool = AgentsTool::new(fake(&scratch, &failing), &data);
    let mut begun = 0;
    let why = tool
        .install("codex", "2.0.0", || begun += 1)
        .expect_err("fails");
    assert_eq!(why, "The download did not match its checksum.");
    assert_eq!(begun, 1, "it had begun before it failed");
    assert!(installed_agents(&data).is_empty());

    let broken = fake(
        &scratch,
        "#!/bin/sh\necho 'No route to the host.' >&2\nexit 1\n",
    );
    let tool = AgentsTool::new(broken, &data);
    assert_eq!(tool.list().expect_err("fails"), "No route to the host.");

    let missing = AgentsTool::new(scratch.join("nowhere/docket-agents"), &data);
    assert_eq!(missing.list().expect_err("not found"), "");
    assert_eq!(missing.install("a", "1", || ()).expect_err("not found"), "");
}

#[test]
fn the_program_is_found_beside_the_running_one_first_then_on_path() {
    let _forks = FORKS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let scratch = Scratch::under(TMP, "agents-tool-find");
    let prefix = scratch.join("prefix");
    let other = scratch.join("other");
    for dir in [&prefix, &other] {
        std::fs::create_dir_all(dir).expect("mkdir");
        std::fs::write(dir.join("docket-agents"), "").expect("file");
    }
    let running = prefix.join("detent");
    let path = other.to_str().expect("utf8");
    assert_eq!(
        find_agents_program(Some(&running), path),
        Some(prefix.join("docket-agents"))
    );
    assert_eq!(
        find_agents_program(None, path),
        Some(other.join("docket-agents"))
    );
    assert_eq!(find_agents_program(None, ""), None);
}
