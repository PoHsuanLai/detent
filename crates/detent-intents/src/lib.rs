//! What an agent, a command line or the launcher may do in Settings, declared once: the intents
//! manifest `org.quire.Settings` (`dist/intents/org.quire.Settings.toml`) and the
//! `IntentProvider1` that answers it. Five actions: open a page, read a key, set a key the
//! schema marks agent-settable (undoable, the undo restores the old value), list the accounts
//! and open the Accounts page (read only: an agent never changes an account). Reading and writing
//! the settings files is `detent-files`; which keys exist and which are settable is
//! `detent-model`; telling the window is a [`Notice`](detent_model::Notice) sent through the
//! [`Outbox`](detent_model::Outbox) the binary hands in. The same bus name
//! also serves `org.freedesktop.Application` at `/org/quire/Settings` (the Desktop Entry
//! spec's D-Bus activation), so a launcher can start Settings, raise it, or open an account.

mod accounts;
mod answer;
mod application;
mod ask;
mod callers;
mod context;
mod journal;
mod limits;
mod manifest;
mod provider;
mod serve;

pub use accounts::{AccountsRead, AccountsSource};
pub use application::{
    APPLICATION_INTERFACE, APPLICATION_PATH, HandOverFault, hand_over, hand_over_session,
};
pub use manifest::{ManifestFault, manifest};
pub use provider::SettingsProvider;
pub use serve::{ServeFault, serve, serve_session, serve_with};
