//! Why a model answered, or why the one the person named could not: the footer under the
//! picker. Both come from inferd's `InferEvent` (`Why` and `Declined`), closed sets of facts, so
//! the words here are the whole of what the footer can say.

use porter_infer::{Declined, DeclinedBecause, InferEvent, Why};

/// One reason the footer shows.
#[derive(Debug, Clone, PartialEq)]
pub enum Reason {
    /// A model was chosen to answer, and why.
    Why(Why),
    /// The model the person named could not answer.
    Declined(Declined),
}

impl Reason {
    /// The reason an `event` carries, if it carries one: only `Why` and `Declined` do.
    pub fn of(event: &InferEvent) -> Option<Reason> {
        match event {
            InferEvent::Why(why) => Some(Reason::Why(why.clone())),
            InferEvent::Declined(declined) => Some(Reason::Declined(declined.clone())),
            _ => None,
        }
    }

    /// One sentence for the footer.
    pub fn line(&self) -> String {
        match self {
            Reason::Why(why) => why_line(why),
            Reason::Declined(declined) => declined_line(declined),
        }
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

#[cfg(test)]
mod tests {
    use super::Reason;
    use porter_core::{AccountId, ModelId};
    use porter_infer::{Declined, DeclinedBecause, InferEvent, InferRefusal, ModelRef, Why};

    fn model(name: &str) -> ModelRef {
        ModelRef {
            account: AccountId::parse("local").expect("an account id"),
            model: ModelId::parse(name).expect("a model id"),
        }
    }

    #[test]
    fn only_why_and_declined_events_carry_a_reason() {
        let cases: [(&str, InferEvent, Option<Reason>); 3] = [
            (
                "why",
                InferEvent::Why(Why::Warm),
                Some(Reason::Why(Why::Warm)),
            ),
            ("text", InferEvent::TextDelta("hello".to_owned()), None),
            (
                "declined",
                InferEvent::Declined(Declined {
                    model: model("holo"),
                    because: DeclinedBecause::NoRoom,
                }),
                Some(Reason::Declined(Declined {
                    model: model("holo"),
                    because: DeclinedBecause::NoRoom,
                })),
            ),
        ];
        for (case, event, want) in cases {
            assert_eq!(Reason::of(&event), want, "{case}");
        }
    }

    #[test]
    fn every_fact_reads_as_one_sentence_naming_the_model_where_it_has_one() {
        let why = |why| Reason::Why(why).line();
        let declined = |because| {
            Reason::Declined(Declined {
                model: model("holo"),
                because,
            })
            .line()
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
}
