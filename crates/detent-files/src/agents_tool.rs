//! The program that reaches the network for coding assistants (`docket-agents`): it lists what
//! the registry offers and downloads one. It is a separate program, run as a child process; this
//! is the only place it is started.

use crate::agents::AGENTS_DIR;
use crate::values::DataHome;
use detent_model::{Candidate, Presence};
use docket_settings::dirs_of;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The file name of the program.
const PROGRAM: &str = "docket-agents";

/// Where the program is: beside the running program (an install prefix's `bin`), else in a
/// directory of `path` (the value of `PATH`).
pub fn find_agents_program(running: Option<&Path>, path: &str) -> Option<PathBuf> {
    running
        .and_then(Path::parent)
        .map(Path::to_owned)
        .into_iter()
        .chain(dirs_of(path))
        .map(|dir| dir.join(PROGRAM))
        .find(|file| file.is_file())
}

/// The first line of `text` that says something.
fn reason_of(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_owned()
}

fn candidate_of(line: &str) -> Option<Candidate> {
    let json: Value = serde_json::from_str(line).ok()?;
    let text = |key: &str| json.get(key)?.as_str().map(str::to_owned);
    let installed = json.get("installed")?.as_bool()?;
    Some(Candidate {
        id: text("id")?,
        name: text("name")?,
        description: text("description").unwrap_or_default(),
        version: text("version")?,
        presence: if installed {
            Presence::Installed
        } else {
            Presence::Online
        },
    })
}

/// `docket-agents` over docket's agents folder.
#[derive(Debug, Clone)]
pub struct AgentsTool {
    program: PathBuf,
    dir: PathBuf,
}

impl AgentsTool {
    /// The program at `program`, working in the agents folder under `data`.
    pub fn new(program: PathBuf, data: &DataHome) -> Self {
        Self {
            program,
            dir: data.0.join(AGENTS_DIR),
        }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.program);
        command
            .arg("--dir")
            .arg(&self.dir)
            .args(args)
            .stdin(Stdio::null());
        command
    }

    /// Every assistant on offer, in the registry's order. Reaches the network and waits for it.
    /// An error is what the program said (empty when it said nothing or did not start).
    pub fn list(&self) -> Result<Vec<Candidate>, String> {
        let out = self
            .command(&["list"])
            .stderr(Stdio::piped())
            .output()
            .map_err(|_| String::new())?;
        if !out.status.success() {
            return Err(reason_of(&String::from_utf8_lossy(&out.stderr)));
        }
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(candidate_of)
            .collect())
    }

    /// Downloads `version` of `id` and waits for it. `begun` is called when the program says it has
    /// started, before it reaches the network. An error is the one sentence the program gave.
    pub fn install(&self, id: &str, version: &str, begun: impl FnOnce()) -> Result<(), String> {
        let mut child = self
            .command(&["install", id, version])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| String::new())?;
        let mut said = String::new();
        if let Some(stdout) = child.stdout.take() {
            let mut lines = BufReader::new(stdout);
            if lines.read_line(&mut String::new()).is_ok_and(|n| n > 0) {
                begun();
            }
            // The rest is only drained, so the program is never left waiting on a full pipe.
            let _ = lines.read_to_end(&mut Vec::new());
        }
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut said);
        }
        match child.wait() {
            Ok(status) if status.success() => Ok(()),
            _ => Err(reason_of(&said)),
        }
    }
}
