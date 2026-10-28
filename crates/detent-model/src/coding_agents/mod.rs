//! The coding agents as a third kind of choice for the assistant's model, beside the models inferd
//! runs: `agent/<program>` in the text slot's rows, in a group of their own, each one either
//! ready or greyed with a plain reason. Choosing one hands the assistant's whole conversation to
//! that agent. Automatic never takes one: its list shows them greyed.
//!
//! inferd owns the model rows and knows nothing of `agents.toml`; docket owns the agents. So the
//! agents are added here, to the rows the module described, from what docket's settings say of
//! them: [`offers`] makes the list, [`with_agents`] lays it over the rows. The words a person reads
//! for them (the group's name, the reasons) come from the caller. Pure.

mod add;
mod add_choice;
mod offer;

#[cfg(test)]
mod add_tests;
#[cfg(test)]
mod pick_tests;
#[cfg(test)]
mod tests;

pub use add::{
    AddForm, AddRefusal, FoundAgent, RemoveQuestion, Request, removal_of, unlisted_found,
};
pub use add_choice::{AddAction, Candidate, Listing, Presence, Step};
pub use offer::{AgentOffer, AgentReason, offers};

use crate::picker::Described;
use ds_settings::schema::{ChoiceGroup, ChoiceWord, KeyKind, KeySpec, UnavailableReason};

/// The reason key of an agent whose switch is off: picking it asks to turn the switch on. A key
/// the picker matches, never shown; the window words `Gate::TurnOn` itself.
pub(crate) const TURNED_OFF: &str = "agents:turned-off";

/// The key the agents' group is stored under: a mark the picker matches, never shown; the
/// window words the group itself.
pub const GROUP_KEY: &str = "agents:group";

/// The reason key of an agent that is only a member of Automatic's list by choosing it directly.
pub const CHOOSE_DIRECTLY_KEY: &str = "agents:choose-directly";

/// A fact `with_agents` lays over the picker's rows, for the window to word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentWord {
    /// The group the agents are listed under.
    Group,
    /// The reason an agent is greyed in Automatic's list.
    ChooseDirectly,
    /// The reason an agent cannot be chosen now.
    Reason(AgentReason),
}

const SIGN_IN_KEY: &str = "agents:sign-in:";

impl AgentWord {
    /// The key this fact is stored under in the picker's schema.
    pub fn key(&self) -> String {
        match self {
            AgentWord::Group => GROUP_KEY.to_owned(),
            AgentWord::ChooseDirectly => CHOOSE_DIRECTLY_KEY.to_owned(),
            AgentWord::Reason(AgentReason::NotInstalled) => "agents:not-installed".to_owned(),
            AgentWord::Reason(AgentReason::NeedsSignIn { label }) => {
                format!("{SIGN_IN_KEY}{label}")
            }
            AgentWord::Reason(AgentReason::NotSetUp) => "agents:not-set-up".to_owned(),
            AgentWord::Reason(AgentReason::TurnedOff) => TURNED_OFF.to_owned(),
        }
    }

    /// The fact a stored key stands for; none for a word the module wrote itself.
    pub fn of_key(key: &str) -> Option<AgentWord> {
        match key {
            GROUP_KEY => Some(AgentWord::Group),
            CHOOSE_DIRECTLY_KEY => Some(AgentWord::ChooseDirectly),
            "agents:not-installed" => Some(AgentWord::Reason(AgentReason::NotInstalled)),
            "agents:not-set-up" => Some(AgentWord::Reason(AgentReason::NotSetUp)),
            TURNED_OFF => Some(AgentWord::Reason(AgentReason::TurnedOff)),
            other => other.strip_prefix(SIGN_IN_KEY).map(|label| {
                AgentWord::Reason(AgentReason::NeedsSignIn {
                    label: label.to_owned(),
                })
            }),
        }
    }
}

/// The prefix of an agent's word.
pub const WORD_PREFIX: &str = "agent/";

/// Which of the picker's keys a spec is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChoiceKind {
    /// A tier of the text slot: the choice itself.
    Choice,
    /// The text slot's Automatic list.
    Pool,
}

fn place_of(path: &str) -> Option<ChoiceKind> {
    let text_slot = |slot: &str| matches!(slot, "text" | "llm");
    if let Some((slot, _tier)) = path
        .strip_prefix("ai.model.")
        .and_then(|r| r.split_once('.'))
        && text_slot(slot)
    {
        return Some(ChoiceKind::Choice);
    }
    path.strip_prefix("ai.auto.pool.")
        .filter(|slot| text_slot(slot))
        .map(|_| ChoiceKind::Pool)
}

/// The menu a spec offers, whichever of the two shapes it is.
fn variants_of(kind: &mut KeyKind) -> Option<&mut Vec<String>> {
    match kind {
        KeyKind::Menu { variants } => Some(variants),
        KeyKind::List(inner) => match inner.as_mut() {
            KeyKind::Menu { variants } => Some(variants),
            _ => None,
        },
        _ => None,
    }
}

fn lay(spec: &mut KeySpec, place: ChoiceKind, offers: &[AgentOffer]) {
    let KeySpec {
        kind,
        labels,
        unavailable,
        groups,
        ..
    } = spec;
    let Some(variants) = variants_of(kind) else {
        return;
    };
    for offer in offers {
        if !variants.contains(&offer.word) {
            variants.push(offer.word.clone());
        }
        labels.0.insert(offer.word.clone(), offer.label.clone());
        let word = ChoiceWord(offer.word.clone());
        groups.insert(word.clone(), ChoiceGroup(AgentWord::Group.key()));
        let reason = match (place, &offer.blocked) {
            (ChoiceKind::Pool, _) => Some(AgentWord::ChooseDirectly.key()),
            (ChoiceKind::Choice, Some(reason)) => Some(AgentWord::Reason(reason.clone()).key()),
            (ChoiceKind::Choice, None) => None,
        };
        match reason {
            Some(reason) => {
                unavailable.insert(word, UnavailableReason(reason));
            }
            None => {
                unavailable.remove(&word);
            }
        }
    }
}

/// `described` with the agents of `offers` laid over the text slot's rows and its Automatic
/// list, each fact as a key for the window to word. The other slots' rows are as the module described them.
pub fn with_agents(mut described: Described, offers: &[AgentOffer]) -> Described {
    if offers.is_empty() {
        return described;
    }
    for spec in &mut described.keys {
        if let Some(place) = place_of(&spec.path.0) {
            lay(spec, place, offers);
        }
    }
    described
}
