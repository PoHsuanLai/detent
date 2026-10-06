//! The manifest this app ships, read from the file the installer puts under
//! `$XDG_DATA_DIRS/quire/intents/`: one declaration, so the file and the provider cannot drift.

use docket_core::{Manifest, ManifestError, ValidManifest, validate};

/// `dist/intents/org.quire.Settings.toml`.
const TEXT: &str = include_str!("../../../dist/intents/org.quire.Settings.toml");

/// The shipped manifest is not usable. A test holds this from happening.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ManifestFault {
    /// The file does not parse as a manifest.
    #[error("the manifest does not parse: {0}")]
    Parse(String),
    /// It parses and breaks a rule of the vocabulary.
    #[error("the manifest is invalid: {0:?}")]
    Invalid(ManifestError),
}

/// The validated manifest of `org.quire.Settings`.
pub fn manifest() -> Result<ValidManifest, ManifestFault> {
    let parsed: Manifest = toml::from_str(TEXT).map_err(|e| ManifestFault::Parse(e.to_string()))?;
    validate(parsed).map_err(ManifestFault::Invalid)
}

#[cfg(test)]
mod tests {
    use super::manifest;

    #[test]
    fn the_shipped_manifest_validates_and_names_its_actions() {
        let manifest = manifest().expect("a valid manifest");
        let m = manifest.manifest();
        assert_eq!(m.app.as_str(), "org.quire.Settings");
        let names: Vec<&str> = m.actions.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "settings.page.show",
                "settings.key.read",
                "settings.key.set",
                "settings.accounts.list",
                "settings.accounts.open"
            ]
        );
    }
}
