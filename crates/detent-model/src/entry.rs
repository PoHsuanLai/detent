//! One key as the app shows it.

use crate::agent::AgentSetting;
use crate::config_file::ConfigFile;
use crate::control::Control;
use ds_settings::schema::KeySpec;

/// A schema key with the file that holds its value and the control that draws it.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyEntry {
    /// The key as its program's schema describes it.
    pub spec: KeySpec,
    /// The file the value lives in.
    pub file: ConfigFile,
    /// What draws it.
    pub control: Control,
    /// Whether an agent may set it.
    pub agent: AgentSetting,
}

impl KeyEntry {
    /// `spec`, held in `file`.
    pub fn new(spec: KeySpec, file: ConfigFile) -> KeyEntry {
        let control = Control::of(&spec);
        KeyEntry {
            spec,
            file,
            control,
            agent: AgentSetting::HandsOff,
        }
    }

    /// This entry with `agent` as its setting.
    #[must_use]
    pub fn with_agent(self, agent: AgentSetting) -> KeyEntry {
        KeyEntry { agent, ..self }
    }
}
