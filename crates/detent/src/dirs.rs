//! Where the schemas and the settings are, read from the environment once, here at the edge.

use detent_files::ConfigHome;
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
    use super::{config_home_from, unique};
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn xdg_config_home_wins_over_home() {
        let cases: &[(Option<&str>, Option<&str>, Option<&str>)] = &[
            (Some("/xdg"), Some("/home/ada"), Some("/xdg")),
            (None, Some("/home/ada"), Some("/home/ada/.config")),
            (None, None, None),
        ];
        for (xdg, home, want) in cases {
            let got = config_home_from(xdg.map(OsString::from), home.map(OsString::from));
            assert_eq!(
                got.map(|h| h.0),
                want.map(PathBuf::from),
                "{xdg:?} {home:?}"
            );
        }
    }

    #[test]
    fn a_directory_listed_twice_is_scanned_once() {
        let dirs = ["/a", "/b", "/a", "/c", "/b"].map(PathBuf::from).to_vec();
        assert_eq!(unique(dirs), ["/a", "/b", "/c"].map(PathBuf::from).to_vec());
    }
}
