//! What each coding agent is to the person choosing: ready, or greyed with why.

use super::WORD_PREFIX;
use docket_settings::{AcpAgents, AgentRow, Agents, Found, Install, SignInState, Source};

/// Why an agent cannot be chosen now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentReason {
    /// Its program is not on this computer.
    NotInstalled,
    /// It asked for a sign-in first; `label` is what the person calls it.
    NeedsSignIn {
        /// The agent's name.
        label: String,
    },
    /// The program is on this computer, but no entry of the agents file runs it.
    NotSetUp,
    /// Coding agents are switched off.
    TurnedOff,
}

/// Whether an agent can be chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentStanding {
    /// It can.
    Ready,
    /// It cannot, and why.
    Blocked(AgentReason),
}

/// One agent as a word of the assistant's menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOffer {
    /// The stored word, `agent/<program>`.
    pub word: String,
    /// What the person reads.
    pub label: String,
    /// Whether it can be chosen.
    pub standing: AgentStanding,
}

/// Whether `program` can be the second half of a word: inferd's id grammar (lower-case letters
/// and digits, then those and `.`, `_`, `-`; at most 64 bytes).
fn is_id(program: &str) -> bool {
    let mut bytes = program.bytes();
    let first = bytes
        .next()
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit());
    first
        && program.len() <= 64
        && bytes.all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        })
}

fn missing(source: &Source) -> bool {
    matches!(
        source,
        Source::Registry {
            installed: Install::NotInstalled,
            ..
        } | Source::Own {
            installed: Install::NotInstalled
        }
    )
}

/// An entry's standing, in the order a person can act on: a program that is not there first,
/// then a sign-in, then the switch.
fn standing_of(row: &AgentRow, switch: AcpAgents) -> AgentStanding {
    let reason = if missing(&row.source) {
        Some(AgentReason::NotInstalled)
    } else if matches!(row.sign_in, SignInState::NeedsSignIn { .. }) {
        Some(AgentReason::NeedsSignIn {
            label: row.label.clone(),
        })
    } else if switch == AcpAgents::Off {
        Some(AgentReason::TurnedOff)
    } else {
        None
    };
    reason.map_or(AgentStanding::Ready, AgentStanding::Blocked)
}

/// The agents the assistant can be handed to, in the file's order, then the known programs found
/// on this computer that no entry runs (greyed: "Not set up yet"). An entry whose name is not
/// an id cannot be a word and is left out.
pub fn offers(agents: &Agents, found: &[Found], switch: AcpAgents) -> Vec<AgentOffer> {
    let listed = agents
        .rows
        .iter()
        .filter(|row| is_id(&row.program))
        .map(|row| AgentOffer {
            word: format!("{WORD_PREFIX}{}", row.program),
            label: row.label.clone(),
            standing: standing_of(row, switch),
        });
    let mut unset: Vec<AgentOffer> = Vec::new();
    for one in docket_settings::unlisted(agents, found) {
        let word = format!("{WORD_PREFIX}{}", one.known.program);
        // Two programs of one agent (its adapter and its own) are one row.
        let named = |offer: &AgentOffer| offer.label == one.known.label;
        if unset.iter().any(named) || agents.rows.iter().any(|row| row.label == one.known.label) {
            continue;
        }
        unset.push(AgentOffer {
            word,
            label: one.known.label.to_owned(),
            standing: AgentStanding::Blocked(AgentReason::NotSetUp),
        });
    }
    listed.chain(unset).collect()
}
