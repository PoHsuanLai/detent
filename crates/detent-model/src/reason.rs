//! Why a model answered, or why the one the person named could not: the footer under the
//! picker. Both come from inferd's `InferEvent` (`Why` and `Declined`), closed sets of facts; the
//! window's words for them are in `detent-ui`'s copy.

use porter_infer::{Declined, InferEvent, Why};

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
}

#[cfg(test)]
mod tests {
    use super::Reason;
    use porter_core::{AccountId, ModelId};
    use porter_infer::{Declined, DeclinedBecause, InferEvent, ModelRef, Why};

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
}
