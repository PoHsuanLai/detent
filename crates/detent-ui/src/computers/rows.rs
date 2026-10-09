//! The rows of inferd's `Candidates` and `Guests`, read into the model's types. A row that does
//! not say what the interface promises is left out.

use detent_model::{Approval, Candidate, Guest, GuestState, NodeKey};
use porter_dbus::{
    CANDIDATE_KEY_MODELS, CANDIDATE_KEY_NAME, CANDIDATE_KEY_NEEDS_APPROVAL, Details,
    GUEST_KEY_NAME, GUEST_KEY_SINCE, GUEST_KEY_STATE,
};

fn text(details: &Details, key: &str) -> Option<String> {
    String::try_from(details.get(key)?.try_clone().ok()?).ok()
}

fn models(details: &Details) -> Vec<String> {
    details
        .get(CANDIDATE_KEY_MODELS)
        .and_then(|value| value.try_clone().ok())
        .and_then(|value| Vec::<(String, String)>::try_from(value).ok())
        .map(|models| models.into_iter().map(|(_, name)| name).collect())
        .unwrap_or_default()
}

/// A candidate from a row of `Candidates`.
pub(super) fn candidate((node, details): (String, Details)) -> Option<Candidate> {
    let needs = details
        .get(CANDIDATE_KEY_NEEDS_APPROVAL)
        .and_then(|value| value.try_clone().ok())
        .and_then(|value| bool::try_from(value).ok())?;
    Some(Candidate {
        node: NodeKey(node),
        name: text(&details, CANDIDATE_KEY_NAME)?,
        models: models(&details),
        approval: if needs {
            Approval::Needed
        } else {
            Approval::Given
        },
    })
}

/// A guest from a row of `Guests`.
pub(super) fn guest((node, details): (String, Details)) -> Option<Guest> {
    let state = match text(&details, GUEST_KEY_STATE)?.as_str() {
        "approved" => GuestState::Approved,
        "denied" => GuestState::Denied,
        "asking" => GuestState::Asking,
        _ => return None,
    };
    let since = details
        .get(GUEST_KEY_SINCE)
        .and_then(|value| value.try_clone().ok())
        .and_then(|value| i64::try_from(value).ok())?;
    Some(Guest {
        node: NodeKey(node),
        name: text(&details, GUEST_KEY_NAME)?,
        state,
        since,
    })
}
