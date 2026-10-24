//! The command line: where the window opens.

use clap::Parser;
use detent_model::PageSlug;
use detent_ui::Opening;
use std::path::{Path, PathBuf};

/// The desktop's Settings app.
#[derive(Debug, Parser)]
#[command(name = "detent", version)]
pub struct Cli {
    /// Open on this page (`appearance`, `dock`, `mouse_and_gestures`, `software-update`, `assistant-model`, ...); `assistant-model-setup` opens the first-run step that asks for the assistant's model, and `accounts-setup` the one that asks the person to sign in to their accounts.
    #[arg(long)]
    page: Option<String>,
    /// Open with this in the search field.
    #[arg(long)]
    search: Option<String>,
    /// Show the one-time welcome window (AI on this computer, which local runtime) and exit; does
    /// nothing when it has already been shown.
    #[arg(long)]
    first_run: bool,
    /// Serve the intents with no window and take no focus: what the bus runs (the D-Bus service
    /// file and the systemd unit) to answer an action. A window opens only when an action asks for
    /// one, and the process exits after a while with none.
    #[arg(long, conflicts_with_all = ["page", "search", "first_run", "write_search_entries"])]
    service: bool,
    /// Write the launcher's search entries for Settings' pages and keys to this file
    /// (`quire/search/org.quire.Settings.toml`) and exit; the installer runs it.
    #[arg(long, value_name = "FILE")]
    write_search_entries: Option<PathBuf>,
    /// With `--write-search-entries`: read schemas from this `quire/settings` folder instead of
    /// the data directories' (repeat for several; an installer names the staged tree).
    #[arg(long, value_name = "DIR", requires = "write_search_entries")]
    schemas: Vec<PathBuf>,
    /// Programs named in this file (the person's own, mode 600 or 644; `[[peer]]` rows with `app`,
    /// `role` and `exe`) are let in as the services they stand for, at a standing below every real
    /// proof: a stand-in built in a checkout is accountd, inferd or syncd for this run only. The
    /// desktop's own table is read as usual and wins. Only a build with the `dev-callers` feature has
    /// this option.
    #[cfg(feature = "dev-callers")]
    #[arg(long, value_name = "FILE")]
    dev_callers: Option<PathBuf>,
}

/// What this run of `detent` is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Run {
    /// The Settings window, opening here.
    Settings(Opening),
    /// The first-run window (the desktop's setup), when it has not been shown yet.
    Setup,
    /// The intents with no window, until an action asks for one.
    Service,
    /// Write the search entries of the schemas in `schemas` (the data directories' when empty)
    /// to `file`.
    SearchEntries {
        file: PathBuf,
        schemas: Vec<PathBuf>,
    },
}

impl Cli {
    /// The rows file `--dev-callers` named; none in a build without the option.
    pub fn dev_callers(&self) -> Option<&Path> {
        #[cfg(feature = "dev-callers")]
        {
            self.dev_callers.as_deref()
        }
        #[cfg(not(feature = "dev-callers"))]
        {
            None
        }
    }

    /// What to run.
    pub fn run(self) -> Run {
        if let Some(file) = self.write_search_entries {
            return Run::SearchEntries {
                file,
                schemas: self.schemas,
            };
        }
        if self.service {
            return Run::Service;
        }
        if self.first_run {
            return Run::Setup;
        }
        Run::Settings(Opening {
            page: self.page.map(PageSlug),
            query: self.search.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Cli, Run};
    use clap::Parser;
    use detent_model::PageSlug;
    use detent_ui::Opening;

    fn opening(args: &[&str]) -> Opening {
        match Cli::try_parse_from(args).expect("valid flags").run() {
            Run::Settings(opening) => opening,
            other => panic!("not a window: {other:?}"),
        }
    }

    #[test]
    fn the_flags_say_where_the_window_opens() {
        // `(arguments, the page, the search)`; no flags open on the first page.
        const OPENINGS: &[(&[&str], Option<&str>, &str)] = &[
            (
                &["detent", "--page", "dock", "--search", "magnification"],
                Some("dock"),
                "magnification",
            ),
            (&["detent"], None, ""),
            (
                &["detent", "--page", "software-update"],
                Some("software-update"),
                "",
            ),
            (
                &["detent", "--page", "assistant-model-setup"],
                Some("assistant-model-setup"),
                "",
            ),
            (
                &["detent", "--page", "accounts-setup"],
                Some("accounts-setup"),
                "",
            ),
        ];
        for (args, page, query) in OPENINGS {
            let opening = opening(args);
            assert_eq!(
                opening.page,
                page.map(|slug| PageSlug(slug.to_owned())),
                "{args:?}"
            );
            assert_eq!(opening.query, *query, "{args:?}");
        }
    }

    #[test]
    fn the_service_flag_is_a_run_with_no_window() {
        let cli = Cli::try_parse_from(["detent", "--service"]).expect("valid flags");
        assert_eq!(cli.run(), Run::Service);
        assert!(Cli::try_parse_from(["detent", "--service", "--page", "dock"]).is_err());
        assert!(Cli::try_parse_from(["detent", "--service", "--first-run"]).is_err());
    }

    #[test]
    fn writing_the_search_entries_names_the_file_and_the_schema_folders() {
        let cli = Cli::try_parse_from([
            "detent",
            "--write-search-entries",
            "/tmp/out.toml",
            "--schemas",
            "/a/quire/settings",
            "--schemas",
            "/b/quire/settings",
        ])
        .expect("valid flags");
        assert_eq!(
            cli.run(),
            Run::SearchEntries {
                file: "/tmp/out.toml".into(),
                schemas: vec!["/a/quire/settings".into(), "/b/quire/settings".into()],
            }
        );
        assert!(Cli::try_parse_from(["detent", "--schemas", "/a"]).is_err());
    }

    #[test]
    fn first_run_is_its_own_run() {
        let cli = Cli::try_parse_from(["detent", "--first-run"]).expect("valid flags");
        assert_eq!(cli.run(), Run::Setup);
    }
}

/// A build with the `dev-callers` feature takes `--dev-callers FILE`; a build without it has no such
/// option, so a release `detent` refuses it like any other flag it does not know.
#[cfg(test)]
mod dev_callers_tests {
    use super::Cli;
    use clap::Parser;
    use std::path::Path;

    #[cfg(feature = "dev-callers")]
    #[test]
    fn this_build_takes_a_dev_callers_file() {
        let cli = Cli::try_parse_from(["detent", "--dev-callers", "/tmp/rows.toml"])
            .expect("valid flags");
        assert_eq!(cli.dev_callers(), Some(Path::new("/tmp/rows.toml")));
        let none = Cli::try_parse_from(["detent"]).expect("valid flags");
        assert_eq!(none.dev_callers(), None);
    }

    #[cfg(not(feature = "dev-callers"))]
    #[test]
    fn this_build_has_no_dev_callers_option() {
        assert!(Cli::try_parse_from(["detent"]).is_ok());
        assert!(Cli::try_parse_from(["detent", "--dev-callers", "/tmp/rows.toml"]).is_err());
        let cli = Cli::try_parse_from(["detent"]).expect("valid flags");
        assert_eq!(cli.dev_callers(), None::<&Path>);
    }
}
