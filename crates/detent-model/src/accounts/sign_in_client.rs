//! The bring-your-own sign-in keys and their secrets: text rows held apart from the accounts. A key
//! is drawn as it is; a secret is write-only, so the pane knows only whether there is one and the
//! text of a secret exists just while it is typed and on its way to the service.

use super::module::ModuleId;
use super::rows::{CLIENTS, PREFIX};
use crate::picker::Described;
use ds_settings::schema::{KeyPath, KeySpec};

/// What a secret row's path ends in, after the path of the key it belongs to.
const SECRET_LEAF: &str = ".secret";

/// What stands in the pane's reads for a secret that is held, so its text is not kept.
const HELD: &str = "set";

/// Whether `path` is a sign-in secret: a client row whose path ends in `.secret`.
fn is_secret_row(path: &KeyPath) -> bool {
    path.0
        .strip_prefix(PREFIX)
        .is_some_and(|id| id.starts_with(CLIENTS) && id.ends_with(SECRET_LEAF))
}

/// `described` with the text of every sign-in secret replaced by a mark that one is held. A
/// secret is write-only: it is read to know whether there is one, and its text goes no further.
pub(super) fn standing_only(mut described: Described) -> Described {
    for (path, text) in &mut described.values {
        if is_secret_row(path) && !text.is_empty() {
            HELD.clone_into(text);
        }
    }
    described
}

/// Whether a sign-in client field can be filled in now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientEntry {
    /// Yes.
    Open,
    /// Not yet: it is a secret, and the sign-in key it belongs to is still empty.
    WaitsForKey,
}

/// The text of a sign-in secret. Its `Debug` is written by hand and never prints the text, so no
/// log line of an input or an effect that carries one can leak it.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    /// The secret `text` makes, without the spaces and line breaks around a pasted one.
    pub fn new(text: &str) -> Secret {
        Secret(text.trim().to_owned())
    }

    /// Whether nothing was typed.
    pub fn is_blank(&self) -> bool {
        self.0.is_empty()
    }

    /// The text, for the one place that sends it to the service.
    pub fn into_text(self) -> String {
        self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(<hidden>)")
    }
}

/// Whether a secret is held. A secret is never read back, so this is all the pane knows of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretStanding {
    /// There is one.
    Set,
    /// There is none.
    NotSet,
}

/// What a sign-in row holds, as far as the window may know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientValue {
    /// A sign-in key, drawn as it is; empty when the build's own is used.
    Text(String),
    /// A secret: only whether there is one.
    Secret(SecretStanding),
}

impl ClientValue {
    /// What the field is keyed on: it is made anew when this changes.
    pub fn key(&self) -> String {
        match self {
            ClientValue::Text(text) => text.clone(),
            ClientValue::Secret(standing) => format!("{standing:?}"),
        }
    }

    /// What was typed into the field of a row holding this, in the form the row takes.
    pub fn typed(&self, text: String) -> ClientText {
        match self {
            ClientValue::Text(_) => ClientText::Plain(text),
            ClientValue::Secret(_) => ClientText::Secret(Secret::new(&text)),
        }
    }
}

/// What was typed into a sign-in row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientText {
    /// Into a key.
    Plain(String),
    /// Into a secret.
    Secret(Secret),
}

/// One bring-your-own sign-in key (or its secret): a text row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInClient {
    /// The module it is in.
    pub module: ModuleId,
    /// The row, `accounts.clients.<issuer>`.
    pub path: KeyPath,
    /// What the module calls it.
    pub label: String,
    /// What the module says about it.
    pub help: String,
    /// What the row holds now.
    pub value: ClientValue,
    /// Whether it can be filled in now.
    pub entry: ClientEntry,
}

/// What an edit of a sign-in row comes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ClientChange {
    /// Nothing: the text is what the row holds, or a secret was left empty.
    Nothing,
    /// The secret was typed before its key.
    KeyFirst,
    /// Write this key.
    Key(String),
    /// Write this secret.
    NewSecret(Secret),
    /// Write the empty secret, which removes it.
    ClearSecret,
}

impl SignInClient {
    /// The row `spec` describes, holding `text` as the service read it. A secret's text is
    /// dropped here: only whether it is empty is kept (`standing_only` has already marked it).
    pub(super) fn read(module: ModuleId, spec: &KeySpec, text: &str) -> SignInClient {
        let value = if is_secret_row(&spec.path) {
            ClientValue::Secret(if text.is_empty() {
                SecretStanding::NotSet
            } else {
                SecretStanding::Set
            })
        } else {
            ClientValue::Text(text.to_owned())
        };
        SignInClient {
            module,
            path: spec.path.clone(),
            label: spec.label.0.clone(),
            help: spec.help.0.clone(),
            value,
            entry: ClientEntry::Open,
        }
    }

    /// The row after `text` was typed into it, and what that asks of the service. A key is
    /// written only when it changed; a secret only when something was typed, and not before its
    /// key.
    pub(super) fn edited(&mut self, text: ClientText) -> ClientChange {
        match (&mut self.value, text) {
            (ClientValue::Text(held), ClientText::Plain(text)) => {
                let text = text.trim().to_owned();
                if *held == text {
                    ClientChange::Nothing
                } else {
                    held.clone_from(&text);
                    ClientChange::Key(text)
                }
            }
            (ClientValue::Secret(standing), ClientText::Secret(secret)) => {
                if secret.is_blank() {
                    ClientChange::Nothing
                } else if self.entry == ClientEntry::WaitsForKey {
                    ClientChange::KeyFirst
                } else {
                    *standing = SecretStanding::Set;
                    ClientChange::NewSecret(secret)
                }
            }
            (ClientValue::Text(_), ClientText::Secret(_))
            | (ClientValue::Secret(_), ClientText::Plain(_)) => ClientChange::Nothing,
        }
    }
}

impl SignInClient {
    /// The row after the person pressed Clear: a held secret is removed (the service takes the
    /// empty text for that); a key, or a secret there is none of, is left alone.
    pub(super) fn cleared(&mut self) -> ClientChange {
        match &mut self.value {
            ClientValue::Secret(standing @ SecretStanding::Set) => {
                *standing = SecretStanding::NotSet;
                ClientChange::ClearSecret
            }
            ClientValue::Secret(SecretStanding::NotSet) | ClientValue::Text(_) => {
                ClientChange::Nothing
            }
        }
    }
}

/// Marks each secret whose key is still empty as not yet fillable.
pub(super) fn wait_for_keys(clients: &mut [SignInClient]) {
    let empty_keys: Vec<&str> = clients
        .iter()
        .filter(|c| c.value == ClientValue::Text(String::new()))
        .map(|c| c.path.0.as_str())
        .collect();
    let waiting: Vec<KeyPath> = clients
        .iter()
        .filter(|c| {
            c.path
                .0
                .strip_suffix(SECRET_LEAF)
                .is_some_and(|key| empty_keys.contains(&key))
        })
        .map(|c| c.path.clone())
        .collect();
    for client in clients.iter_mut() {
        if waiting.contains(&client.path) {
            client.entry = ClientEntry::WaitsForKey;
        }
    }
}
