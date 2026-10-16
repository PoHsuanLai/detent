//! How to get llama.cpp on this computer: the one command to copy, or where to get it when this
//! computer's distro has no package.

use ds_helpers::{Family, OsRelease};

/// What the person reads when there is no package to install.
pub const RELEASES_LINE: &str = "Get llama.cpp's server from its releases page";

/// How to get llama.cpp here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallAdvice {
    /// A command to run in a terminal.
    Command(String),
    /// No package exists here: the releases page is where the server comes from.
    ReleasesPage,
}

/// A release number, `26.04` as (26, 4) and `14` as (14, 0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Version(u32, u32);

impl Version {
    /// The number `text` writes; `None` for anything else.
    fn of(text: &str) -> Option<Version> {
        let mut parts = text.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next().map_or(Some(0), |part| part.parse().ok())?;
        Some(Version(major, minor))
    }
}

/// The first Debian release that packages llama.cpp.
const DEBIAN_FROM: Version = Version(14, 0);

/// The first Ubuntu release that packages it.
const UBUNTU_FROM: Version = Version(26, 4);

/// Whether `release` is at least `from`; a release with no number counts as `when_unnumbered`.
fn at_least(release: &OsRelease, from: Version, when_unnumbered: Unnumbered) -> bool {
    match release.version_id.as_deref() {
        None => matches!(when_unnumbered, Unnumbered::New),
        Some(text) => Version::of(text).is_some_and(|version| version >= from),
    }
}

/// How a release with no `VERSION_ID` counts.
#[derive(Clone, Copy)]
enum Unnumbered {
    /// As new: Debian testing carries none and is ahead of every stable release.
    New,
    /// As old: no package can be promised.
    Old,
}

/// The package that holds llama.cpp's server on `family`, when this release has one.
fn package(family: Family, release: &OsRelease) -> Option<&'static str> {
    match family {
        Family::Dnf | Family::Pacman => Some("llama-cpp"),
        Family::Apt => {
            let packaged = match release.id.as_str() {
                "debian" => at_least(release, DEBIAN_FROM, Unnumbered::New),
                "ubuntu" => at_least(release, UBUNTU_FROM, Unnumbered::Old),
                _ => false,
            };
            packaged.then_some("llama.cpp-tools")
        }
        Family::Zypper => None,
    }
}

/// How to get llama.cpp on the computer whose `os-release` file holds `os_release` (empty when
/// there is none): the family's install command with the package that holds it, or the releases
/// page where this release has no package, the family is unknown, or openSUSE.
pub fn install_advice(os_release: &str) -> InstallAdvice {
    let release = OsRelease::parse(os_release);
    Family::detect(os_release)
        .and_then(|family| {
            package(family, &release).map(|package| {
                InstallAdvice::Command(format!("{} {package}", family.install_command()))
            })
        })
        .unwrap_or(InstallAdvice::ReleasesPage)
}
