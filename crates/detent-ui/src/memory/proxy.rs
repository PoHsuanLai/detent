//! The caller's side of `org.quire.Memory1.Control`, the members Settings uses.

use std::collections::HashMap;

/// The memory service's name on the bus.
pub(super) const MEMORY_NAME: &str = "org.quire.Memory1";

/// The controls memory offers the person (`org.quire.Memory1.Control`).
#[zbus::proxy(
    interface = "org.quire.Memory1.Control",
    default_service = "org.quire.Memory1",
    default_path = "/org/quire/Memory1"
)]
pub(crate) trait MemoryControl {
    /// Settles a removed Space's memories and history; `removal` is JSON, the reply says what moved.
    fn remove_space(&self, space: &str, removal: &str) -> zbus::Result<String>;

    /// The person agreed to these looser settings (key to value). Only Settings may say so.
    fn confirm_settings(&self, values: HashMap<String, String>) -> zbus::Result<()>;

    /// The looser settings in the file that are not confirmed: key, value waiting, shipped value.
    fn pending_settings(&self) -> zbus::Result<Vec<(String, String, String)>>;

    /// Whether memory keeps itself to its own folders: `confined`, `partial`, `not-available` or
    /// (in a test build only) `off`. It does not change while memory runs.
    #[zbus(property)]
    fn confinement(&self) -> zbus::Result<String>;
}
