//! One key as the app shows it.

use crate::config_file::ConfigFile;
use crate::control::Control;
use crate::extensions::Offers;
use ds_settings::schema::{AgentSetting, KeyKind, KeySpec};

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

    /// This entry with the choices of `offers` in it when its key is a `Provided` one (a look):
    /// the spec takes them as a menu of words and labels (`KeySpec::offering`) and the control
    /// holds the words. Any other entry comes back as it is.
    pub fn offering(&self, offers: &Offers) -> KeyEntry {
        match &self.spec.kind {
            KeyKind::Provided { capability } => {
                let offer = offers.of(capability);
                KeyEntry {
                    spec: self.spec.offering(offer.words.clone(), offer.labels),
                    file: self.file.clone(),
                    control: Control::Provided(offer.words),
                }
            }
            _ => self.clone(),
        }
    }

    /// Whether an agent may set the key: the mark of the schema that supplied it, which
    /// `ds_settings::schema::discover` admitted and `Schema::from_toml` checked (a key under a
    /// never-settable prefix cannot carry the mark); `discover` has already cleared it for a schema
    /// from a folder the person can write.
    pub fn agent(&self) -> AgentSetting {
        self.spec.agent
    }
}
