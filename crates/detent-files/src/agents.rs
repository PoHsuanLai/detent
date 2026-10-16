//! The outside agents docket may start (`docket/agents.toml`), read with what each last offered,
//! and the person's choice of model or way of signing in written back into the entry.

use crate::error::FilesError;
use crate::read::text_of;
use crate::values::{ConfigHome, DataHome};
use docket_agents::AgentsDir;
use docket_settings::{AgentChoice, Agents, ChoiceFault, read_agents, write_agent_choice};

/// The agents file, under the configuration directory.
const AGENTS_FILE: &str = "docket/agents.toml";

/// Where docket keeps what it installed and what each agent offered, under the data directory.
const AGENTS_DIR: &str = "docket/agents";

/// The agents `docket/agents.toml` lists, none when there is no file.
pub fn load_agents(config: &ConfigHome, data: &DataHome) -> Agents {
    // A file that cannot be read lists no agents; docket's own writer refuses to write over it.
    let text = text_of(&config.0.join(AGENTS_FILE))
        .ok()
        .flatten()
        .unwrap_or_default();
    read_agents(&text, &AgentsDir::at(&data.0.join(AGENTS_DIR)))
}

/// Sets the person's `choice` in the entry of `program`; nothing else in the file changes.
pub fn save_agent_choice(
    config: &ConfigHome,
    program: &str,
    choice: &AgentChoice,
) -> Result<(), FilesError> {
    write_agent_choice(&config.0.join(AGENTS_FILE), program, choice).map_err(|fault| match fault {
        ChoiceFault::Io => FilesError::Io {
            path: AGENTS_FILE.into(),
            source: std::io::Error::other(fault.to_string()),
        },
        _ => FilesError::Garbled {
            path: AGENTS_FILE.into(),
            reason: fault.to_string(),
        },
    })
}
