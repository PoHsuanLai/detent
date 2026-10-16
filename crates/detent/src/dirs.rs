//! Where the schemas and the settings are, read from the environment once, here at the edge.

use detent_files::{ConfigHome, DataHome, StateHome};
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

#[cfg(test)]
mod tests {
    use super::{config_home_from, data_home_from, state_home_from, unique};
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
    fn a_directory_listed_twice_is_scanned_once() {
        let dirs = ["/a", "/b", "/a", "/c", "/b"].map(PathBuf::from).to_vec();
        assert_eq!(unique(dirs), ["/a", "/b", "/c"].map(PathBuf::from).to_vec());
    }
}
