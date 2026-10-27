//! The update service's properties, as one `GetAll` answer read into [`UpdateWire`].
//!
//! One answer is one moment: eight reads, one property at a time, could mix the old offer's
//! version with the new one's notes when the service moved between them.

use detent_model::{Absence, UpdateWire};
use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

/// The property `name` of the answer as the type the service documents, or the service not
/// making sense.
fn field<'a, T: TryFrom<&'a OwnedValue>>(
    all: &'a HashMap<String, OwnedValue>,
    name: &str,
) -> Result<T, Absence> {
    all.get(name)
        .and_then(|value| T::try_from(value).ok())
        .ok_or(Absence::Failed)
}

/// The properties of `org.quire.Updates1` in `all`.
pub fn wire_of(all: &HashMap<String, OwnedValue>) -> Result<UpdateWire, Absence> {
    let text = |name: &str| field::<&str>(all, name).map(str::to_owned);
    Ok(UpdateWire {
        state: text("State")?,
        current: text("Current")?,
        available: text("Available")?,
        notes: text("Notes")?,
        size: field(all, "Size")?,
        progress: field(all, "Progress")?,
        last_checked: field(all, "LastChecked")?,
        problem: text("Problem")?,
        // An older service has no ProblemKind, and a kind of another type is not a kind this window
        // knows: both read as none, which is a plain failure.
        problem_kind: text("ProblemKind").unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::wire_of;
    use std::collections::HashMap;
    use zbus::zvariant::{OwnedValue, Value};

    fn owned(value: impl Into<Value<'static>>) -> OwnedValue {
        OwnedValue::try_from(value.into()).expect("a plain value")
    }

    fn all() -> HashMap<String, OwnedValue> {
        HashMap::from([
            ("State".to_owned(), owned("available")),
            ("Current".to_owned(), owned("0.3.1")),
            ("Available".to_owned(), owned("0.4.0")),
            ("Notes".to_owned(), owned("Faster start.")),
            ("Size".to_owned(), owned(45_600_000_u64)),
            ("Progress".to_owned(), owned(250_u32)),
            ("LastChecked".to_owned(), owned(1_000_i64)),
            ("Problem".to_owned(), owned("")),
            ("ProblemKind".to_owned(), owned("version-changed")),
        ])
    }

    #[test]
    fn one_answer_reads_into_every_property() {
        let wire = wire_of(&all()).expect("a wire");
        assert_eq!(wire.state, "available");
        assert_eq!(wire.available, "0.4.0");
        assert_eq!(wire.notes, "Faster start.");
        assert_eq!(wire.size, 45_600_000);
        assert_eq!(wire.progress, 250);
        assert_eq!(wire.last_checked, 1_000);
        assert_eq!(wire.problem_kind, "version-changed");
    }

    #[test]
    fn a_property_missing_or_of_another_type_is_a_service_that_makes_no_sense() {
        let mut missing = all();
        missing.remove("Notes");
        assert!(wire_of(&missing).is_err());
        let mut odd = all();
        odd.insert("Size".to_owned(), owned("big"));
        assert!(wire_of(&odd).is_err());
        // An older service has no kind, and an odd one is no kind: neither is a service that
        // makes no sense, both are a plain failure.
        let mut old = all();
        old.remove("ProblemKind");
        assert_eq!(wire_of(&old).expect("a wire").problem_kind, "");
        let mut strange = all();
        strange.insert("ProblemKind".to_owned(), owned(7_u32));
        assert_eq!(wire_of(&strange).expect("a wire").problem_kind, "");
    }
}
