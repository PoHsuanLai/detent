//! Where the schemas and the settings are, read from the environment once, here at the edge.

use detent_files::{CacheHome, ConfigHome, DataHome, StateHome};
use std::ffi::OsString;
use std::path::PathBuf;

/// `value` as an absolute path; unset, empty and relative values are no value. The XDG rule, as
/// quire applies it to its own directories (`ds_settings::xdg::base_dir`, which is crate-private:
/// when quire publishes it this file calls it), so `XDG_CONFIG_HOME=` in a unit file is not "the
/// working directory".
fn absolute(value: Option<OsString>) -> Option<PathBuf> {
    value
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

/// `xdg` when it is a usable path, else `home` (itself usable) with `under_home` below it.
fn base_from(xdg: Option<OsString>, home: Option<OsString>, under_home: &str) -> Option<PathBuf> {
    absolute(xdg).or_else(|| absolute(home).map(|home| home.join(under_home)))
}

/// `$XDG_CONFIG_HOME`, else `$HOME/.config`.
fn config_home_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<ConfigHome> {
    base_from(xdg, home, ".config").map(ConfigHome)
}

/// The person's config home, `None` when neither variable is usable.
pub fn config_home() -> Option<ConfigHome> {
    config_home_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

/// `$XDG_STATE_HOME`, else `$HOME/.local/state`.
fn state_home_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<StateHome> {
    base_from(xdg, home, ".local/state").map(StateHome)
}

/// The person's state home, `None` when neither variable is usable.
pub fn state_home() -> Option<StateHome> {
    state_home_from(std::env::var_os("XDG_STATE_HOME"), std::env::var_os("HOME"))
}

/// `$XDG_DATA_HOME`, else `$HOME/.local/share`.
fn data_home_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<DataHome> {
    base_from(xdg, home, ".local/share").map(DataHome)
}

/// The person's data home, `None` when neither variable is usable.
pub fn data_home() -> Option<DataHome> {
    data_home_from(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
}

/// `$XDG_CACHE_HOME`, else `$HOME/.cache`.
fn cache_home_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<CacheHome> {
    base_from(xdg, home, ".cache").map(CacheHome)
}

/// The person's cache home, `None` when neither variable is usable.
pub fn cache_home() -> Option<CacheHome> {
    cache_home_from(std::env::var_os("XDG_CACHE_HOME"), std::env::var_os("HOME"))
}

/// `dirs` without a directory already listed, in order: `$XDG_DATA_HOME` is often also in
/// `$XDG_DATA_DIRS`, and its schemas would otherwise shadow themselves.
fn unique(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    dirs.into_iter().fold(Vec::new(), |mut kept, dir| {
        if !kept.contains(&dir) {
            kept.push(dir);
        }
        kept
    })
}

/// Each `$XDG_DATA_DIRS` entry's `quire/settings`, then `$XDG_DATA_HOME`'s: the order of trust, a
/// schema in the person's own directory adds a program and never replaces an installed one's.
pub fn schema_dirs() -> Vec<PathBuf> {
    unique(ds_settings::schema::data_dirs())
}

/// Each data directory's `quire/intents`, the folders of the installed apps' manifests, in the
/// order of `schema_dirs` (beside which they sit).
pub fn intent_dirs() -> Vec<PathBuf> {
    intent_dirs_beside(&schema_dirs())
}

/// The `intents` folder beside each of the `quire/settings` folders `schemas`.
fn intent_dirs_beside(schemas: &[PathBuf]) -> Vec<PathBuf> {
    schemas
        .iter()
        .filter_map(|settings| settings.parent().map(|quire| quire.join("intents")))
        .collect()
}

/// Where the app stores export the desktop entries of what they install, after the folders the
/// data directories name: Flatpak's system install, then snap's.
const STORE_EXPORTS: [&str; 2] = [
    "/var/lib/flatpak/exports/share/applications",
    "/var/lib/snapd/desktop/applications",
];

/// The `applications` folders of the desktop entries: the data home's, each of the data
/// directories' (`/usr/local/share:/usr/share` when the variable is unset), then Flatpak's user
/// install and the stores' exports, in the order of precedence.
fn application_dirs_from(data_home: Option<DataHome>, data_dirs: Option<OsString>) -> Vec<PathBuf> {
    let listed = data_dirs
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| OsString::from("/usr/local/share:/usr/share"));
    let shares = data_home
        .iter()
        .map(|home| home.0.clone())
        .chain(std::env::split_paths(&listed).filter(|dir| dir.is_absolute()));
    let exports = data_home
        .iter()
        .map(|home| home.0.join("flatpak/exports/share/applications"))
        .chain(STORE_EXPORTS.map(PathBuf::from));
    unique(
        shares
            .map(|share| share.join("applications"))
            .chain(exports)
            .collect(),
    )
}

/// The folders desktop entries are read from, at the edge.
pub fn application_dirs() -> Vec<PathBuf> {
    application_dirs_from(data_home(), std::env::var_os("XDG_DATA_DIRS"))
}

#[cfg(test)]
mod tests {
    use super::{
        application_dirs_from, cache_home_from, config_home_from, data_home_from,
        intent_dirs_beside, state_home_from, unique,
    };
    use detent_files::DataHome;
    use std::ffi::OsString;
    use std::path::PathBuf;

    type Home = fn(Option<OsString>, Option<OsString>) -> Option<PathBuf>;

    #[test]
    fn the_xdg_variable_wins_over_home() {
        let config: Home = |xdg, home| config_home_from(xdg, home).map(|h| h.0);
        let state: Home = |xdg, home| state_home_from(xdg, home).map(|h| h.0);
        let data: Home = |xdg, home| data_home_from(xdg, home).map(|h| h.0);
        let cache: Home = |xdg, home| cache_home_from(xdg, home).map(|h| h.0);
        // `(directory, XDG variable, HOME, the home)`.
        type Case<'a> = (
            &'a str,
            Home,
            Option<&'a str>,
            Option<&'a str>,
            Option<&'a str>,
        );
        let cases: &[Case] = &[
            (
                "config",
                config,
                Some("/xdg"),
                Some("/home/ada"),
                Some("/xdg"),
            ),
            (
                "config",
                config,
                None,
                Some("/home/ada"),
                Some("/home/ada/.config"),
            ),
            ("config", config, None, None, None),
            (
                "state",
                state,
                Some("/xdg"),
                Some("/home/ada"),
                Some("/xdg"),
            ),
            (
                "state",
                state,
                None,
                Some("/home/ada"),
                Some("/home/ada/.local/state"),
            ),
            ("state", state, None, None, None),
            ("data", data, Some("/xdg"), Some("/home/ada"), Some("/xdg")),
            (
                "data",
                data,
                None,
                Some("/home/ada"),
                Some("/home/ada/.local/share"),
            ),
            ("data", data, None, None, None),
            (
                "cache",
                cache,
                Some("/xdg"),
                Some("/home/ada"),
                Some("/xdg"),
            ),
            (
                "cache",
                cache,
                None,
                Some("/home/ada"),
                Some("/home/ada/.cache"),
            ),
            ("cache", cache, None, None, None),
            // An empty or relative variable is no variable (it is not the working directory).
            (
                "config",
                config,
                Some(""),
                Some("/home/ada"),
                Some("/home/ada/.config"),
            ),
            (
                "state",
                state,
                Some("state"),
                Some("/home/ada"),
                Some("/home/ada/.local/state"),
            ),
            ("data", data, Some(""), None, None),
            ("config", config, None, Some(""), None),
            ("config", config, None, Some("home"), None),
        ];
        for (name, function, xdg, home, want) in cases {
            let got = function(xdg.map(OsString::from), home.map(OsString::from));
            assert_eq!(got, want.map(PathBuf::from), "{name} {xdg:?} {home:?}");
        }
    }

    #[test]
    fn manifests_are_looked_for_beside_each_schema_folder_in_its_order() {
        let schemas = [
            "/usr/share/quire/settings",
            "/home/ada/.local/share/quire/settings",
        ]
        .map(PathBuf::from)
        .to_vec();
        assert_eq!(
            intent_dirs_beside(&schemas),
            [
                "/usr/share/quire/intents",
                "/home/ada/.local/share/quire/intents"
            ]
            .map(PathBuf::from)
            .to_vec()
        );
    }

    #[test]
    fn a_directory_listed_twice_is_scanned_once() {
        let dirs = ["/a", "/b", "/a", "/c", "/b"].map(PathBuf::from).to_vec();
        assert_eq!(unique(dirs), ["/a", "/b", "/c"].map(PathBuf::from).to_vec());
    }

    #[test]
    fn desktop_entries_are_looked_for_in_the_data_home_then_the_data_directories_then_the_stores() {
        let home = Some(DataHome(PathBuf::from("/home/ada/.local/share")));
        let dirs =
            |value: Option<&str>| application_dirs_from(home.clone(), value.map(OsString::from));
        let paths = |list: &[&str]| list.iter().map(PathBuf::from).collect::<Vec<_>>();
        assert_eq!(
            dirs(Some("/opt/share:/usr/share")),
            paths(&[
                "/home/ada/.local/share/applications",
                "/opt/share/applications",
                "/usr/share/applications",
                "/home/ada/.local/share/flatpak/exports/share/applications",
                "/var/lib/flatpak/exports/share/applications",
                "/var/lib/snapd/desktop/applications",
            ])
        );
        assert_eq!(
            dirs(None)[1..3],
            paths(&["/usr/local/share/applications", "/usr/share/applications"])[..],
            "unset means the standard directories"
        );
        assert_eq!(
            dirs(Some(""))[1],
            PathBuf::from("/usr/local/share/applications")
        );
        assert!(
            !dirs(Some("relative:/usr/share")).contains(&PathBuf::from("relative/applications")),
            "a relative directory is not the working directory"
        );
        assert_eq!(
            application_dirs_from(None, Some(OsString::from("/usr/share"))),
            paths(&[
                "/usr/share/applications",
                "/var/lib/flatpak/exports/share/applications",
                "/var/lib/snapd/desktop/applications",
            ]),
            "no data home, no user folders"
        );
    }
}
