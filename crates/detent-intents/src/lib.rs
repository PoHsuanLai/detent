//! What an agent, a command line or the launcher may do in Settings, declared once: the intents
//! manifest `org.quire.Settings` (`dist/intents/org.quire.Settings.toml`) and the
//! `IntentProvider1` that answers it. Three actions: open a page, read a key, and set a key the
//! schema marks agent-settable (undoable, the undo restores the old value). Reading and writing
//! the settings files is `detent-files`; which keys exist and which are settable is
//! `detent-model`; telling the window is a [`Window`] the binary implements.

mod answer;
mod ask;
mod context;
mod journal;
mod manifest;
mod provider;
mod serve;
mod window;

pub use manifest::{ManifestFault, manifest};
pub use provider::SettingsProvider;
pub use serve::{ServeFault, serve, serve_session};
pub use window::{Window, WindowGone};
