//! The Extensions installed on this computer: what the Look menu offers, what the Extensions
//! page lists, and the switch that turns one off in `quire/extensions.toml`. The folder scan, the
//! checks and the on and off state are quire's (`ds-extensions`); this reads them for the window
//! and writes the one file.

use crate::error::FilesError;
use crate::read::text_of;
use crate::save::save_keys;
use crate::values::{ConfigHome, DataHome};
use detent_model::{
    ConfigFile, ExtensionKeep, ExtensionList, ExtensionPower, ExtensionRow, ExtensionStanding,
    ExtensionTrouble, InstalledExtensions, Offer, Offers,
};
use ds_extensions::bayonet::manifest::Raw;
use ds_extensions::bayonet::registry::Candidate;
use ds_extensions::{
    Capability, ExtensionId, ExtensionsFile, Found, Power, Problem, Removal, Search,
    YOUR_CHANGES_ID, YourChanges, choices_for,
};
use ds_settings::schema::{FilePath, KeyPath};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// The file that lists the Extensions turned off, under the config home.
const FILE: &str = "quire/extensions.toml";

/// The key of that file that holds the list.
const OFF_KEY: &str = "extensions.off";

/// The capabilities a `Provided` key can name, by the word its schema writes, and the kind of
/// Extension that serves each. A `Provided` key naming any other word offers only Standard.
const DATA_WORDS: [(&str, Capability); 2] = [
    ("look", Capability::Look),
    ("file_icons", Capability::FileIcons),
];

/// Where Extensions are looked for, and where the list of the ones turned off is kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionHome {
    search: Search,
    config: ConfigHome,
}

impl ExtensionHome {
    /// The Extensions in `data` (the person's own) and in each of the `system` data directories,
    /// with the turned-off list under `config`.
    pub fn under(config: &ConfigHome, data: &DataHome, system: &[PathBuf]) -> ExtensionHome {
        ExtensionHome {
            search: Search::new(&data.0, system),
            config: config.clone(),
        }
    }

    /// What is installed now: one look at the folders and the turned-off list. Nothing here
    /// fails: a manifest that cannot be read is listed as one that cannot be used.
    pub fn read(&self) -> InstalledExtensions {
        let found = self.search.scan(&self.off_ids());
        InstalledExtensions {
            offers: offers_of(&found),
            list: list_of(&found, keep_of(&self.search)),
        }
    }

    /// Turns the Extension `id` on or off. Every other line of the file stays as it was.
    pub fn set_power(&self, id: &str, power: ExtensionPower) -> Result<(), FilesError> {
        let file =
            ConfigFile::parse(&FilePath(FILE.to_owned())).map_err(|reason| FilesError::Io {
                path: PathBuf::from(FILE),
                source: std::io::Error::other(format!("{reason:?}")),
            })?;
        let wanted = match power {
            ExtensionPower::On => Power::On,
            ExtensionPower::Off => Power::Off,
        };
        let settings = self
            .file()
            .extensions
            .with(&ExtensionId(id.to_owned()), wanted);
        let rows = settings
            .off
            .iter()
            .map(|row| {
                let mut table = toml::Table::new();
                table.insert("id".to_owned(), toml::Value::String(row.id.clone()));
                toml::Value::Table(table)
            })
            .collect();
        save_keys(
            &self.config,
            &file,
            &[(&KeyPath(OFF_KEY.to_owned()), &toml::Value::Array(rows))],
        )
    }

    /// The file as it is. One that is missing or that cannot be understood turns nothing off.
    fn file(&self) -> ExtensionsFile {
        let path = self.config.0.join(FILE);
        text_of(&path)
            .ok()
            .flatten()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    fn off_ids(&self) -> Vec<ExtensionId> {
        self.file().off_ids()
    }
}

/// What each `Provided` capability offers: Standard, then the usable Extensions that provide it.
fn offers_of(found: &Found) -> Offers {
    DATA_WORDS
        .iter()
        .fold(Offers::default(), |offers, (word, capability)| {
            let resolved = found.resolve(*capability);
            let choices = choices_for(*capability, resolved.registry.installed());
            offers.with(
                word,
                Offer {
                    words: choices.words().to_vec(),
                    labels: choices.labels().clone(),
                },
            )
        })
}

/// The capability words `candidate` has an entry for.
fn words_of(candidate: &Candidate<Raw>) -> Vec<String> {
    candidate
        .manifest
        .provides
        .iter()
        .map(|entry| entry.word().to_owned())
        .collect()
}

/// Whether the person's own changes block taking their Extension away: they do while it holds
/// any, and a folder that cannot be read counts as holding some (quire's rule).
fn keep_of(search: &Search) -> ExtensionKeep {
    match YourChanges::of(search).map(|changes| changes.removal()) {
        Some(Removal::Blocked) => ExtensionKeep::Holds,
        Some(Removal::Allowed) | None => ExtensionKeep::Free,
    }
}

/// A row for `candidate`, standing as `standing`. The person's changes get `own`, the rest are free.
fn row_of(
    candidate: &Candidate<Raw>,
    standing: ExtensionStanding,
    own: ExtensionKeep,
) -> ExtensionRow {
    let keep = if candidate.manifest.id.as_str() == YOUR_CHANGES_ID {
        own
    } else {
        ExtensionKeep::Free
    };
    ExtensionRow {
        id: candidate.manifest.id.as_str().to_owned(),
        name: candidate.manifest.name.clone(),
        summary: candidate.manifest.summary.clone(),
        provides: words_of(candidate),
        standing,
        keep,
    }
}

/// Every Extension found: the ones in use, the ones turned off and the ones that cannot be used.
/// An id found in two places is one row, the first copy (the person's own comes first).
fn list_of(found: &Found, own: ExtensionKeep) -> ExtensionList {
    let problems: Vec<Problem> = DATA_WORDS
        .iter()
        .flat_map(|(_, capability)| found.resolve(*capability).problems())
        .collect();
    let standing_of = |id: &str| {
        problems.iter().find(|problem| problem.id.0 == id).map_or(
            ExtensionStanding::On,
            |problem| {
                ExtensionStanding::Unusable(ExtensionTrouble::Reported(problem.words().to_owned()))
            },
        )
    };
    let on = found
        .candidates
        .iter()
        .map(|candidate| row_of(candidate, standing_of(candidate.manifest.id.as_str()), own));
    let off = found.off.iter().filter_map(|off| {
        off.candidates
            .first()
            .map(|candidate| row_of(candidate, ExtensionStanding::Off, own))
    });
    let unread = found.rejected.iter().filter_map(|rejected| {
        let folder = rejected.file.parent()?.file_name()?.to_str()?;
        Some(ExtensionRow {
            id: folder.to_owned(),
            name: folder.to_owned(),
            summary: None,
            provides: Vec::new(),
            standing: ExtensionStanding::Unusable(ExtensionTrouble::Unreadable),
            keep: ExtensionKeep::Free,
        })
    });
    let mut seen = BTreeSet::new();
    ExtensionList::of(
        on.chain(off)
            .chain(unread)
            .filter(|row| seen.insert(row.id.clone()))
            .collect(),
    )
}
