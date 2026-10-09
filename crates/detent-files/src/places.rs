//! The built-in assistant's places, in docket's settings file: read as docket reads them, and
//! written one key of the `assistant` table at a time, so every other key of the file keeps its
//! text.

use crate::error::FilesError;
use crate::save::save;
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

/// The places docket's settings file holds; none when there is no file.
pub fn load_places(home: &ConfigHome) -> PlacesLoaded {
    let text = file()
        .ok()
        .and_then(|file| std::fs::read_to_string(home.path_of(&file)).ok())
        .unwrap_or_default();
    read_places(&text)
}

/// Writes `places` into docket's settings file: each key of the `assistant` table, and nothing
/// else in the file.
pub fn save_places(home: &ConfigHome, places: &Places) -> Result<(), FilesError> {
    let file = file()?;
    let table: toml::Table =
        write_places(places)
            .parse()
            .map_err(|e: toml::de::Error| FilesError::Garbled {
                path: SETTINGS_FILE.into(),
                reason: e.to_string(),
            })?;
    let Some(toml::Value::Table(assistant)) = table.get(PLACES_PATH) else {
        return Ok(());
    };
    assistant.iter().try_for_each(|(key, value)| {
        save(home, &file, &KeyPath(format!("{PLACES_PATH}.{key}")), value)
    })
}
