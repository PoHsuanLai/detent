//! The menus of an outside agent's row: the model it runs and the way it signs in, as the agent
//! last offered them, and what a pick writes; and the one list of assistants that joins what
//! docket records of each agent with what accountd knows of its sign-in.

use crate::accounts::Account;
use docket_settings::{AgentChoice, AgentRow, Offers, Pick, SignInState, Source};

/// The menu word for "the agent's own choice": no model is written.
pub const AGENTS_OWN: &str = "";

/// One entry of a menu: the word written, and the name read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// What the file holds (empty for the agent's own).
    pub word: String,
    /// What the person reads.
    pub name: String,
}

fn entry(word: &str, name: &str) -> Entry {
    Entry {
        word: word.to_owned(),
        name: name.to_owned(),
    }
}

/// The models `row` can run: its own choice first, then what it offered, then the one the file
/// names when the agent no longer offers it (so the menu never hides what is written).
pub fn model_menu(row: &AgentRow) -> Vec<Entry> {
    let mut menu = vec![entry(AGENTS_OWN, "The agent\u{2019}s own choice")];
    if let Offers::Seen { models, .. } = &row.offers {
        menu.extend(models.iter().map(|m| entry(&m.id, &m.name)));
    }
    if let Some(word) = model_word(row).filter(|word| menu.iter().all(|e| e.word != *word)) {
        menu.push(entry(&word, &word));
    }
    menu
}

/// The word of the model the file names; the agent's own when it names none.
pub fn model_word(row: &AgentRow) -> Option<String> {
    match &row.model {
        docket_settings::ModelState::AgentsOwn => None,
        docket_settings::ModelState::Chosen { id, .. } => Some(id.clone()),
    }
}

/// The ways `row` listed for signing in, none unless it asked for a sign-in.
pub fn way_menu(row: &AgentRow) -> Vec<Entry> {
    match &row.sign_in {
        SignInState::NeedsSignIn { ways } => ways.iter().map(|w| entry(&w.id, &w.name)).collect(),
        SignInState::NotSeenYet | SignInState::SignedIn | SignInState::Unknown => Vec::new(),
    }
}

/// What picking `word` in the model menu writes.
pub fn model_choice(word: &str) -> AgentChoice {
    AgentChoice {
        model: if word == AGENTS_OWN {
            Pick::Clear
        } else {
            Pick::Set(word.to_owned())
        },
        way: Pick::Keep,
    }
}

/// What picking `word` in the way menu writes.
pub fn way_choice(word: &str) -> AgentChoice {
    AgentChoice {
        model: Pick::Keep,
        way: Pick::Set(word.to_owned()),
    }
}

/// One outside assistant as the person sees it: one row, whichever of the two sources know it.
#[derive(Debug, Clone, PartialEq)]
pub struct Assistant {
    /// What the person reads.
    pub name: String,
    /// What docket records of the agent: how it is installed, the model, the sign-in it saw.
    pub agent: Option<AgentRow>,
    /// What accountd knows: how the sign-in stands now, and the way to its account.
    pub account: Option<Account>,
}

/// Whether the account named `label`, with id `key` and provider `provider`, is the agent's.
fn is_same(agent: &AgentRow, label: &str, key: &str, provider: Option<&str>) -> bool {
    let registry = match &agent.source {
        Source::Registry { id, .. } => Some(id.as_str()),
        Source::Own { .. } => None,
    };
    label.trim().eq_ignore_ascii_case(agent.label.trim())
        || [Some(key), provider]
            .into_iter()
            .flatten()
            .any(|id| id == agent.program || Some(id) == registry)
}

/// The assistants: each agent docket lists, joined with its account where there is one, then the
/// accounts no agent claims.
pub fn assistants_of(agents: &[AgentRow], accounts: &[Account]) -> Vec<Assistant> {
    let claims = |agent: &AgentRow, account: &Account| {
        let provider = account.provider.as_ref().map(|p| p.id.as_str());
        is_same(agent, &account.label, &account.key.0, provider)
    };
    let joined = agents.iter().map(|agent| Assistant {
        name: agent.label.clone(),
        agent: Some(agent.clone()),
        account: accounts
            .iter()
            .find(|account| claims(agent, account))
            .cloned(),
    });
    let alone = accounts
        .iter()
        .filter(|account| !agents.iter().any(|agent| claims(agent, account)))
        .map(|account| Assistant {
            name: account.label.clone(),
            agent: None,
            account: Some(account.clone()),
        });
    joined.chain(alone).collect()
}

#[cfg(test)]
mod tests {
    use super::is_same;
    use docket_settings::{AgentRow, ModelState, Offers, SignInState, Source};

    fn claude() -> AgentRow {
        AgentRow {
            program: "claude-code-bin".to_owned(),
            label: "Claude Code".to_owned(),
            source: Source::Registry {
                id: "claude-code".to_owned(),
                version: "1.0.0".to_owned(),
                installed: docket_settings::Install::Installed,
            },
            command: None,
            model: ModelState::AgentsOwn,
            offers: Offers::NotSeenYet,
            sign_in: SignInState::NotSeenYet,
            way: None,
            rewind: docket_core::Rewind::Agent,
        }
    }

    #[test]
    fn an_account_is_the_agents_by_name_by_program_or_by_registry_id() {
        // `(case, label, key, provider, is it the agent's)`.
        const CASES: &[(&str, &str, &str, Option<&str>, bool)] = &[
            (
                "the same name in other letters",
                " claude code ",
                "x",
                None,
                true,
            ),
            (
                "the program as the key",
                "Other",
                "claude-code-bin",
                None,
                true,
            ),
            (
                "the registry id as the provider",
                "Other",
                "x",
                Some("claude-code"),
                true,
            ),
            (
                "another assistant",
                "Antigravity",
                "agy",
                Some("google"),
                false,
            ),
        ];
        for (case, label, key, provider, want) in CASES {
            assert_eq!(is_same(&claude(), label, key, *provider), *want, "{case}");
        }
    }
}
