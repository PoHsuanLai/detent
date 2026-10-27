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
    AddForm, AddRefusal, FoundAgent, RemoveAnswer, RemoveQuestion, Request, removal_of,
    unlisted_found,
};
pub use add_choice::{AddAction, Candidate, Listing, Presence, Step};
pub use offer::{AgentOffer, AgentReason, AgentStanding, offers};

use crate::picker::Described;
use ds_settings::schema::{ChoiceGroup, ChoiceWord, KeyKind, KeySpec, UnavailableReason};

/// The reason key of an agent whose switch is off: picking it asks to turn the switch on. A key
/// the picker matches, never shown; the window words `Gate::TurnOn` itself.
pub(crate) const TURNED_OFF: &str = "agents:turned-off";

/// A word of the window `with_agents` needs from its caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentWord<'a> {
    /// The name of the group the agents are listed under.
    Group,
    /// The reason an agent is greyed in Automatic's list.
    ChooseDirectly,
    /// The reason an agent cannot be chosen now.
    Reason(&'a AgentReason),
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

fn lay(
    spec: &mut KeySpec,
    place: ChoiceKind,
    offers: &[AgentOffer],
    says: &dyn Fn(AgentWord<'_>) -> String,
) {
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
        groups.insert(word.clone(), ChoiceGroup(says(AgentWord::Group)));
        let reason = match (place, &offer.standing) {
            (ChoiceKind::Pool, _) => Some(says(AgentWord::ChooseDirectly)),
            (ChoiceKind::Choice, AgentStanding::Blocked(AgentReason::TurnedOff)) => {
                Some(TURNED_OFF.to_owned())
            }
            (ChoiceKind::Choice, AgentStanding::Blocked(reason)) => {
                Some(says(AgentWord::Reason(reason)))
            }
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
/// list, their words from `says`. The other slots' rows are as the module described them.
pub fn with_agents(
    mut described: Described,
    offers: &[AgentOffer],
    says: &dyn Fn(AgentWord<'_>) -> String,
) -> Described {
    if offers.is_empty() {
        return described;
    }
    for spec in &mut described.keys {
        if let Some(place) = place_of(&spec.path.0) {
            lay(spec, place, offers, says);
        }
    }
    described
}
