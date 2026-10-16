//! The built-in assistant's places, in docket's settings file: read as docket reads them, and
//! written as the `assistant` table's keys in one edit, so every other key of the file keeps its
//! text.

use crate::error::FilesError;
use crate::read::text_of;
use crate::save::save_keys;
use crate::values::ConfigHome;
use detent_model::ConfigFile;
use docket_settings::{
    PLACES_PATH, Places, PlacesLoaded, SETTINGS_FILE, read_places, write_places,
};
use ds_settings::schema::{FilePath, KeyPath};

fn file() -> Result<ConfigFile, FilesError> {
    ConfigFile::parse(&FilePath(SETTINGS_FILE.to_owned())).map_err(|reason| FilesError::Garbled {
        path: SETTINGS_FILE.into(),
        reason: format!("{reason:?}"),
    })
}

/// The places docket's settings file holds; none when there is no file (or it cannot be read:
/// saving then refuses, with the reason, rather than writing over it).
pub fn load_places(home: &ConfigHome) -> PlacesLoaded {
    let text = file()
        .ok()
        .and_then(|file| text_of(&home.path_of(&file)).ok().flatten())
        .unwrap_or_default();
    read_places(&text)
}

/// Writes `places` into docket's settings file: each key of the `assistant` table in one edit
/// and one rename (docket's directory watch never sees some of the keys changed and not the
/// others), and nothing else in the file.
pub fn save_places(home: &ConfigHome, places: &Places) -> Result<(), FilesError> {
    let file = file()?;
    let table: toml::Table =
        write_places(places)
            .parse()
            .map_err(|e: toml::de::Error| FilesError::Garbled {
                path: SETTINGS_FILE.into(),
                reason: e.message().to_owned(),
            })?;
    let Some(toml::Value::Table(assistant)) = table.get(PLACES_PATH) else {
        return Err(FilesError::Io {
            path: SETTINGS_FILE.into(),
            source: std::io::Error::other("the places could not be written"),
        });
    };
    let keys: Vec<(KeyPath, &toml::Value)> = assistant
        .iter()
        .map(|(key, value)| (KeyPath(format!("{PLACES_PATH}.{key}")), value))
        .collect();
    let edits: Vec<(&KeyPath, &toml::Value)> =
        keys.iter().map(|(key, value)| (key, *value)).collect();
    save_keys(home, &file, &edits)
}
