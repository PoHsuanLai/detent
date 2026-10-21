//! The command line: where the window opens.

use clap::Parser;
use detent_model::PageSlug;
use detent_ui::Opening;
use std::path::Path;
#[cfg(feature = "dev-callers")]
use std::path::PathBuf;

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
    /// The first-run window, when it has not been shown yet.
    FirstRun,
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
        if self.first_run {
            return Run::FirstRun;
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
            Run::FirstRun => panic!("not the first run"),
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
    fn first_run_is_its_own_run() {
        let cli = Cli::try_parse_from(["detent", "--first-run"]).expect("valid flags");
        assert_eq!(cli.run(), Run::FirstRun);
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
