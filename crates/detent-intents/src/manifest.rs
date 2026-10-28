//! The manifest this app ships, read from the file the installer puts under
//! `$XDG_DATA_DIRS/quire/intents/`: one declaration, so the file and the provider cannot drift.

use docket_manifest::{Manifest, ManifestError, ValidManifest, validate};

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
    use prov::Effect;

    #[test]
    fn the_shipped_manifest_validates_and_names_its_actions_with_their_effects() {
        let manifest = manifest().expect("a valid manifest");
        let m = manifest.manifest();
        assert_eq!(m.app.as_str(), "org.quire.Settings");
        // Nothing offers to change an account or an `ai.*` row: only a key may be set, and it
        // can be taken back.
        let actions: Vec<(&str, Effect)> = m
            .actions
            .iter()
            .map(|a| (a.name.as_str(), a.effect))
            .collect();
        assert_eq!(
            actions,
            [
                ("settings.page.show", Effect::Read),
                ("settings.page.open", Effect::Read),
                ("settings.key.open", Effect::Read),
                ("settings.key.read", Effect::Read),
                ("settings.key.set", Effect::UndoableWrite),
                ("settings.accounts.list", Effect::Read),
                ("settings.accounts.open", Effect::Read),
                ("settings.intelligence.open", Effect::Read),
            ]
        );
    }
}
