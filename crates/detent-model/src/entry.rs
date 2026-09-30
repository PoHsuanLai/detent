//! One key as the app shows it.

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
}

impl KeyEntry {
    /// `spec`, held in `file`.
    pub fn new(spec: KeySpec, file: ConfigFile) -> KeyEntry {
        let control = Control::of(&spec);
        KeyEntry {
            spec,
            file,
            control,
        }
    }
}
