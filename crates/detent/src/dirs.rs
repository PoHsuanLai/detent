//! Where the schemas and the settings are, read from the environment once, here at the edge.

use detent_files::{ConfigHome, DataHome, StateHome};
use std::ffi::OsString;
use std::path::PathBuf;

/// `$XDG_CONFIG_HOME`, else `$HOME/.config`.
fn config_home_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<ConfigHome> {
    xdg.map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".config")))
        .map(ConfigHome)
}

/// The person's config home, `None` when neither variable is set.
pub fn config_home() -> Option<ConfigHome> {
    config_home_from(
        std::env::var_os("XDG_CONFIG_HOME"),
        std::env::var_os("HOME"),
    )
}

/// `$XDG_STATE_HOME`, else `$HOME/.local/state`.
fn state_home_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<StateHome> {
    xdg.map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".local/state")))
        .map(StateHome)
}

/// The person's state home, `None` when neither variable is set.
pub fn state_home() -> Option<StateHome> {
    state_home_from(std::env::var_os("XDG_STATE_HOME"), std::env::var_os("HOME"))
}

/// `$XDG_DATA_HOME`, else `$HOME/.local/share`.
fn data_home_from(xdg: Option<OsString>, home: Option<OsString>) -> Option<DataHome> {
    xdg.map(PathBuf::from)
        .or_else(|| home.map(|home| PathBuf::from(home).join(".local/share")))
        .map(DataHome)
}

/// The person's data home, `None` when neither variable is set.
pub fn data_home() -> Option<DataHome> {
    data_home_from(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
}

/// Where keycap keeps the machine's keymap: `$KEYCAP_CONFIG_DIR`, else `/etc/keycap` (keycap's
/// own default).
fn keycap_dir_from(var: Option<OsString>) -> PathBuf {
    var.filter(|dir| !dir.is_empty())
        .map_or_else(|| PathBuf::from("/etc/keycap"), PathBuf::from)
}

/// The directory of this computer's keymap.
pub fn keycap_config_dir() -> PathBuf {
    keycap_dir_from(std::env::var_os("KEYCAP_CONFIG_DIR"))
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

/// `$XDG_DATA_HOME/quire/settings`, then each `$XDG_DATA_DIRS` entry's.
pub fn schema_dirs() -> Vec<PathBuf> {
    unique(ds_settings::schema::data_dirs())
}

#[cfg(test)]
mod tests {
    use super::{config_home_from, data_home_from, keycap_dir_from, state_home_from, unique};
    use std::ffi::OsString;
    use std::path::PathBuf;

    type Home = fn(Option<OsString>, Option<OsString>) -> Option<PathBuf>;

    #[test]
    fn the_xdg_variable_wins_over_home() {
        let config: Home = |xdg, home| config_home_from(xdg, home).map(|h| h.0);
        let state: Home = |xdg, home| state_home_from(xdg, home).map(|h| h.0);
        let data: Home = |xdg, home| data_home_from(xdg, home).map(|h| h.0);
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
        ];
        for (name, function, xdg, home, want) in cases {
            let got = function(xdg.map(OsString::from), home.map(OsString::from));
            assert_eq!(got, want.map(PathBuf::from), "{name} {xdg:?} {home:?}");
        }
    }

    #[test]
    fn keycaps_directory_is_the_variable_or_its_default() {
        let cases: &[(Option<&str>, &str)] = &[
            (Some("/scratch/keycap"), "/scratch/keycap"),
            (Some(""), "/etc/keycap"),
            (None, "/etc/keycap"),
        ];
        for (var, want) in cases {
            let got = keycap_dir_from(var.map(OsString::from));
            assert_eq!(got, PathBuf::from(want), "{var:?}");
        }
    }

    #[test]
    fn a_directory_listed_twice_is_scanned_once() {
        let dirs = ["/a", "/b", "/a", "/c", "/b"].map(PathBuf::from).to_vec();
        assert_eq!(unique(dirs), ["/a", "/b", "/c"].map(PathBuf::from).to_vec());
    }
}
