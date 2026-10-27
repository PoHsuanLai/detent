//! What a row of an account is: the kind of row and the account it belongs to, told from its
//! path and kind alone.

use super::model::{AccountKey, SwitchState};
use super::module::ModuleId;
use crate::picker::Described;
use ds_settings::schema::{KeyKind, KeyPath, KeySpec};
use porter_core::SignInWay;

/// The prefix of every row's path.
pub(super) const PREFIX: &str = "accounts.";

/// Where the bring-your-own client ids sit: `accounts.clients.<issuer>`.
pub(super) const CLIENTS: &str = "clients.";

/// Between an account's id and a service's kind in a row's path.
const SERVICE: &str = ".service.";

/// After an account's id, in the path of a sync switch: `accounts.<id>.sync.files`, `.sync.photos`.
const SYNC: &str = ".sync.";

/// The path of an account's "Show in Files" switch, after its id: `accounts.<id>.storage.files`.
const SHOW_IN_FILES: &str = ".storage.files";

/// The leaves a sync switch ends in.
const SYNC_LEAVES: [&str; 2] = ["files", "photos"];

/// Between an account's id and an app's number in the path of a row that names an app using it.
const USED_BY: &str = ".used_by.";

/// Between an account's id and `month` or `cap` in the path of a spend row.
const SPEND: &str = ".spend.";

/// The path of syncd's row that starts "Continue as" for a new account.
pub(super) const SYNC_ADD: &str = "accounts.sync.add";

/// Between an account's id and a grant's id in a row's path.
const GRANT: &str = ".grant.";

/// The kind of service a language model is, in a service toggle's path.
pub(super) const LLM: &str = "llm";

/// What a row of an account is, by its kind and where its path puts it.
pub(super) enum Row<'a> {
    Label(&'a str),
    State(&'a str),
    Place(&'a str),
    Expires(&'a str),
    SignIn(&'a str),
    Provider(&'a str),
    Group(&'a str),
    GrantSince(&'a str, &'a str),
    GrantScope(&'a str, &'a str),
    UsedBy(&'a str),
    SpendMonth(&'a str),
    SpendCap(&'a str),
    AccountService(&'a str, SwitchState),
    Sync(SwitchState),
    ShowInFiles(SwitchState),
    Grant,
    Reauth,
    SignOut,
    Remove,
}

/// The account a row belongs to and what the row is. An id may hold dots, so the row's kind
/// and the words between the id and the rest decide, not a count of segments.
pub(super) fn classify<'a>(
    module: ModuleId,
    spec: &'a KeySpec,
    described: &Described,
) -> Option<(AccountKey, Row<'a>)> {
    let rest = spec.path.0.strip_prefix(PREFIX)?;
    let key = |id: &str| AccountKey::of(module, id);
    let held = |path: &KeyPath| {
        SwitchState::of_word(described.values.get(path).map_or("", String::as_str))
    };
    match &spec.kind {
        KeyKind::Toggle { .. } => {
            if let Some((id, kind)) = rest.rsplit_once(SERVICE) {
                return Some((key(id), Row::AccountService(kind, held(&spec.path))));
            }
            if let Some(id) = rest.strip_suffix(SHOW_IN_FILES) {
                return Some((key(id), Row::ShowInFiles(held(&spec.path))));
            }
            let (id, leaf) = rest.rsplit_once(SYNC)?;
            SYNC_LEAVES
                .contains(&leaf)
                .then(|| (key(id), Row::Sync(held(&spec.path))))
        }
        KeyKind::Fixed { variant } => {
            if let Some((id, tail)) = rest.split_once(GRANT) {
                let row = match tail.rsplit_once('.')? {
                    (grant, "since") => Row::GrantSince(grant, variant),
                    (grant, "scope") => Row::GrantScope(grant, variant),
                    _ => return None,
                };
                return Some((key(id), row));
            }
            if let Some((id, leaf)) = rest.split_once(SPEND) {
                let row = match leaf {
                    "month" => Row::SpendMonth(variant),
                    "cap" => Row::SpendCap(variant),
                    _ => return None,
                };
                return Some((key(id), row));
            }
            if let Some((id, _)) = rest.split_once(USED_BY) {
                return Some((key(id), Row::UsedBy(variant)));
            }
            let (id, leaf) = rest.rsplit_once('.')?;
            let row = match leaf {
                "group" => Row::Group(variant),
                "label" => Row::Label(variant),
                "state" => Row::State(variant),
                "place" => Row::Place(variant),
                "expires" => Row::Expires(variant),
                "sign_in" => Row::SignIn(variant),
                "provider" => Row::Provider(variant),
                _ => return None,
            };
            Some((key(id), row))
        }
        KeyKind::Live { .. } => {
            if let Some((id, _)) = rest.split_once(GRANT) {
                return Some((key(id), Row::Grant));
            }
            let (id, leaf) = rest.rsplit_once('.')?;
            let row = match leaf {
                "reauth" => Row::Reauth,
                "sign_out" => Row::SignOut,
                "remove" => Row::Remove,
                _ => return None,
            };
            Some((key(id), row))
        }
        _ => None,
    }
}

pub(super) fn action_name(spec: &KeySpec) -> String {
    match &spec.kind {
        KeyKind::Live { action } => action.label.0.clone(),
        _ => spec.label.0.clone(),
    }
}

/// The way the `sign_in` row's slug names; none for a word this build does not know.
pub(super) fn sign_in_of(word: &str) -> Option<SignInWay> {
    toml::Value::String(word.to_owned()).try_into().ok()
}
