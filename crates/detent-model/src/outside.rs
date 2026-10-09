//! The menus of an outside agent's row: the model it runs and the way it signs in, as the agent
//! last offered them, and what a pick writes.

use docket_settings::{AgentChoice, AgentRow, Offers, Pick, SignInState};

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
