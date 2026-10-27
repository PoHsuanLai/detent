//! The model picker's words: why a model answered, what a choice still needs, what a row says
//! while a pick applies, and the line under a word that cannot be picked as it is.

use detent_model::external::{Declined, DeclinedBecause, Why};
use detent_model::{
    AgentReason, AgentWord, ChoiceGap, Gate, Loading, PickLabel, PoolHeading, Reason,
};

/// What the menu calls the choice that clears a row.
pub(crate) const OFF: &str = "Off";

/// What the menu calls the models this computer runs.
pub(crate) const LOCAL: &str = "Local";

/// What a menu choice is called.
pub(crate) fn pick_label(label: &PickLabel) -> String {
    match label {
        PickLabel::Off => OFF.to_owned(),
        PickLabel::Local => LOCAL.to_owned(),
        PickLabel::Named(name) => name.clone(),
    }
}

/// The header over the coding agents in a menu.
pub(crate) const CODING_AGENTS: &str = "Coding Agents";

/// One sentence for the footer under the picker.
pub(crate) fn reason(reason: &Reason) -> String {
    match reason {
        Reason::Why(why) => why_line(why),
        Reason::Declined(declined) => declined_line(declined),
    }
}

fn why_line(why: &Why) -> String {
    match why {
        Why::Named => "You chose this model.".to_owned(),
        Why::OnlyOne => "The only model that can do this.".to_owned(),
        Why::Nearest => "It runs closer to you than the others.".to_owned(),
        Why::Warm => "Already running.".to_owned(),
        Why::Smallest => "The lightest model that can do this.".to_owned(),
        Why::LeastCost => "The others cost more to use.".to_owned(),
        Why::CatalogueOrder => "First in your list.".to_owned(),
        Why::SharedWithHelper => "Another helper is already using it.".to_owned(),
        Why::FallbackFrom { model } => {
            format!("{} couldn't do this.", model.model)
        }
        Why::Evicted { model } => {
            format!("Loading it, and unloading {} to make room.", model.model)
        }
        Why::Reached { .. } => "Reached through an online account.".to_owned(),
    }
}

fn declined_line(declined: &Declined) -> String {
    let model = &declined.model.model;
    match &declined.because {
        DeclinedBecause::NotListed => {
            format!("{model} can't do this: it isn't offered for this request.")
        }
        DeclinedBecause::Blocked { .. } => {
            format!("{model} can't do this: your privacy settings don't allow it.")
        }
        DeclinedBecause::NotInstalled => {
            format!("{model} can't do this: it isn't on this computer yet.")
        }
        DeclinedBecause::Unavailable => format!("{model} can't do this: it can't run right now."),
        DeclinedBecause::NoRoom => format!("{model} can't do this: it doesn't fit in memory."),
        // A reason a newer porter adds: no more is known than that the model can't answer.
        _ => format!("{model} can't do this."),
    }
}

/// The sentence that tells the person what to do next.
pub(crate) fn gap(gap: ChoiceGap) -> &'static str {
    match gap {
        ChoiceGap::Choice => "Choose a model.",
        ChoiceGap::PoolEmpty => "Turn on at least one model.",
        ChoiceGap::PoolGone => "The models you turned on are not available now. Turn on another.",
        ChoiceGap::NoLocalEngine => "Nothing on this computer is set up to run models yet.",
        ChoiceGap::LocalEngine => "Choose how to run models here.",
        ChoiceGap::LocalModel => "Choose a model to run here.",
    }
}

/// What a row shows while a set applies.
pub(crate) fn loading(loading: &Loading) -> String {
    match loading {
        Loading::Applying => "Applying".to_owned(),
        Loading::Model(name) => format!("Loading {name}\u{2026}"),
    }
}

/// The line shown under a word that cannot be picked as it is; none for one that can.
pub(crate) fn gate_note(gate: &Gate) -> Option<String> {
    match gate {
        Gate::Open => None,
        Gate::NeedsAccount => Some("Add an account to use".to_owned()),
        Gate::Blocked(reason) => Some(blocked_line(reason)),
        Gate::TurnOn => Some("Turned off".to_owned()),
    }
}

/// Why a word is blocked: the module's own sentence, or the words for a coding agent's reason.
pub(crate) fn blocked_line(reason: &str) -> String {
    match AgentWord::of_key(reason) {
        Some(word) => agent_word(&word),
        None => reason.to_owned(),
    }
}

/// The words over a run of Automatic's list; none over the ungrouped lines.
pub(crate) fn pool_heading(heading: &PoolHeading) -> Option<&str> {
    match heading {
        PoolHeading::Unnamed => None,
        PoolHeading::Named(name) => Some(name),
        PoolHeading::Agents => Some(CODING_AGENTS),
        PoolHeading::NoLongerOffered => Some("No longer available"),
    }
}

/// What the picker says when it is told the module refused a change to Automatic's list.
pub(crate) fn pool_refused() -> &'static str {
    "Automatic could not use that model, so its list is unchanged."
}

/// The words `with_agents` lays over the rows of the text slot.
pub(crate) fn agent_word(word: &AgentWord) -> String {
    match word {
        AgentWord::Group => CODING_AGENTS.to_owned(),
        AgentWord::ChooseDirectly => "Choose it directly".to_owned(),
        AgentWord::Reason(reason) => match reason {
            AgentReason::NotInstalled => "Not installed".to_owned(),
            AgentReason::NeedsSignIn { label } => {
                format!("Sign in to {label} first")
            }
            AgentReason::NotSetUp => "Not set up yet".to_owned(),
            AgentReason::TurnedOff => "Turned off".to_owned(),
        },
    }
}

/// The words "Runs with".
pub(crate) const RUNS_WITH: &str = "Runs with";

/// The words "Models".
pub(crate) const MODELS: &str = "Models";

/// The words "Choose from".
pub(crate) const CHOOSE_FROM: &str = "Choose from";

/// The words "No longer available. Turn off to remove.".
pub(crate) const GONE_TURN_OFF: &str = "No longer available. Turn off to remove.";

/// The words "Models on this computer".
pub(crate) const MODELS_ON_THIS_COMPUTER: &str = "Models on this computer";

/// The words "Nothing is running here yet. Ollama, llama.cpp and LM Studio appear here when they are.".
pub(crate) const NOTHING_RUNNING_HERE: &str = "Nothing is running here yet. Ollama, llama.cpp and LM Studio appear \
    here when they are.";

/// The words "Coding agents are turned off. Turn them on to let the assistant hand y".
pub(crate) fn coding_agents_off(name: impl std::fmt::Display) -> String {
    format!(
        "Coding agents are turned off. Turn them on to let the assistant hand your requests to \
         {name}. The desktop still checks what {name} does and asks you when it matters."
    )
}

/// The words "Turn on coding agents?".
pub(crate) const TURN_ON_CODING_AGENTS: &str = "Turn on coding agents?";

#[cfg(test)]
mod tests {
    use super::*;
    use detent_model::external::{AccountId, InferRefusal, ModelId, ModelRef};

    fn model(name: &str) -> ModelRef {
        ModelRef {
            account: AccountId::parse("local").expect("an account id"),
            model: ModelId::parse(name).expect("a model id"),
        }
    }

    #[test]
    fn every_fact_reads_as_one_sentence_naming_the_model_where_it_has_one() {
        let why = |why| reason(&Reason::Why(why));
        let declined = |because| {
            reason(&Reason::Declined(Declined {
                model: model("holo"),
                because,
            }))
        };
        let cases: [(&str, String, &str); 14] = [
            ("named", why(Why::Named), "You chose this model."),
            (
                "only one",
                why(Why::OnlyOne),
                "The only model that can do this.",
            ),
            (
                "nearest",
                why(Why::Nearest),
                "It runs closer to you than the others.",
            ),
            ("warm", why(Why::Warm), "Already running."),
            (
                "smallest",
                why(Why::Smallest),
                "The lightest model that can do this.",
            ),
            (
                "least cost",
                why(Why::LeastCost),
                "The others cost more to use.",
            ),
            ("catalogue", why(Why::CatalogueOrder), "First in your list."),
            (
                "fallback",
                why(Why::FallbackFrom {
                    model: model("qwen"),
                }),
                "qwen couldn't do this.",
            ),
            (
                "evicted",
                why(Why::Evicted {
                    model: model("qwen"),
                }),
                "Loading it, and unloading qwen to make room.",
            ),
            (
                "not listed",
                declined(DeclinedBecause::NotListed),
                "holo can't do this: it isn't offered for this request.",
            ),
            (
                "blocked",
                declined(DeclinedBecause::Blocked {
                    refusal: InferRefusal::Denied,
                }),
                "holo can't do this: your privacy settings don't allow it.",
            ),
            (
                "not installed",
                declined(DeclinedBecause::NotInstalled),
                "holo can't do this: it isn't on this computer yet.",
            ),
            (
                "unavailable",
                declined(DeclinedBecause::Unavailable),
                "holo can't do this: it can't run right now.",
            ),
            (
                "no room",
                declined(DeclinedBecause::NoRoom),
                "holo can't do this: it doesn't fit in memory.",
            ),
        ];
        for (case, got, want) in cases {
            assert_eq!(got, want, "{case}");
        }
    }

    #[test]
    fn a_gap_says_what_to_do_next() {
        const CASES: &[(ChoiceGap, &str)] = &[
            (ChoiceGap::Choice, "Choose a model."),
            (ChoiceGap::PoolEmpty, "Turn on at least one model."),
            (
                ChoiceGap::PoolGone,
                "The models you turned on are not available now. Turn on another.",
            ),
            (
                ChoiceGap::NoLocalEngine,
                "Nothing on this computer is set up to run models yet.",
            ),
            (ChoiceGap::LocalEngine, "Choose how to run models here."),
            (ChoiceGap::LocalModel, "Choose a model to run here."),
        ];
        for (gap_of, want) in CASES {
            assert_eq!(gap(*gap_of), *want);
        }
    }

    #[test]
    fn a_row_says_loading_for_a_model_and_applying_for_the_words_that_name_none() {
        assert_eq!(loading(&Loading::Applying), "Applying");
        assert_eq!(
            loading(&Loading::Model("holo".to_owned())),
            "Loading holo\u{2026}"
        );
    }

    #[test]
    fn a_word_that_cannot_be_picked_as_it_is_has_a_line() {
        let cases = [
            (Gate::Open, None),
            (Gate::NeedsAccount, Some("Add an account to use".to_owned())),
            (
                Gate::Blocked("Not installed".to_owned()),
                Some("Not installed".to_owned()),
            ),
            (
                Gate::Blocked("agents:not-set-up".to_owned()),
                Some("Not set up yet".to_owned()),
            ),
            (Gate::TurnOn, Some("Turned off".to_owned())),
        ];
        for (gate, want) in cases {
            assert_eq!(gate_note(&gate), want, "{gate:?}");
        }
    }

    #[test]
    fn a_run_of_the_list_is_headed_by_its_group_or_by_what_is_gone() {
        let named = PoolHeading::Named("Anthropic".to_owned());
        assert_eq!(pool_heading(&PoolHeading::Unnamed), None);
        assert_eq!(pool_heading(&named), Some("Anthropic"));
        assert_eq!(pool_heading(&PoolHeading::Agents), Some("Coding Agents"));
        assert_eq!(
            pool_heading(&PoolHeading::NoLongerOffered),
            Some("No longer available")
        );
    }

    #[test]
    fn the_agents_are_greyed_with_a_reason_a_person_can_act_on() {
        let cases = [
            (AgentWord::Group, "Coding Agents".to_owned()),
            (AgentWord::ChooseDirectly, "Choose it directly".to_owned()),
            (
                AgentWord::Reason(AgentReason::NotInstalled),
                "Not installed".to_owned(),
            ),
            (
                AgentWord::Reason(AgentReason::NeedsSignIn {
                    label: "Codex".to_owned(),
                }),
                "Sign in to Codex first".to_owned(),
            ),
            (
                AgentWord::Reason(AgentReason::NotSetUp),
                "Not set up yet".to_owned(),
            ),
            (
                AgentWord::Reason(AgentReason::TurnedOff),
                "Turned off".to_owned(),
            ),
        ];
        for (word, want) in cases {
            assert_eq!(agent_word(&word), want);
        }
    }
}
