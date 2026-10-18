//! The coding agents as a third kind of choice for the assistant's model, beside the models inferd
//! runs: `agent/<program>` in the text slot's rows, in a group of their own, each one either
//! ready or greyed with a plain reason. Choosing one hands the assistant's whole conversation to
//! that agent. Automatic never takes one: its list shows them greyed, "Choose it directly".
//!
//! inferd owns the model rows and knows nothing of `agents.toml`; docket owns the agents. So the
//! words are added here, to the rows the module described, from what docket's settings say of the
//! agents: [`offers`] makes the list, [`with_agents`] lays it over the rows. Pure.

mod offer;

#[cfg(test)]
mod tests;

pub use offer::{AgentOffer, AgentReason, AgentStanding, offers};

use crate::picker::Described;
use ds_settings::schema::{ChoiceGroup, ChoiceWord, KeyKind, KeySpec, UnavailableReason};

/// The group the agents are listed under.
pub const GROUP: &str = "Coding Agents";

/// The reason an agent is greyed in Automatic's list.
pub const CHOOSE_DIRECTLY: &str = "Choose it directly";

/// The reason of an agent whose switch is off: picking it asks to turn the switch on.
pub const TURNED_OFF: &str = "Turned off";

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
        groups.insert(word.clone(), ChoiceGroup(GROUP.to_owned()));
        let reason = match (place, &offer.standing) {
            (ChoiceKind::Pool, _) => Some(CHOOSE_DIRECTLY.to_owned()),
            (ChoiceKind::Choice, AgentStanding::Blocked(reason)) => Some(reason.words()),
            (ChoiceKind::Choice, AgentStanding::Ready) => None,
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
/// list. The other slots' rows are as the module described them.
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
