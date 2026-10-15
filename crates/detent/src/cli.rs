//! The command line: where the window opens.

use clap::Parser;
use detent_model::PageSlug;
use detent_ui::Opening;

/// The desktop's Settings app.
#[derive(Debug, Parser)]
#[command(name = "detent", version)]
pub struct Cli {
    /// Open on this page (`appearance`, `dock`, `mouse_and_gestures`, `software-update`, `assistant-model`, ...); `assistant-model-setup` opens the first-run step that asks for the assistant's model.
    #[arg(long)]
    page: Option<String>,
    /// Open with this in the search field.
    #[arg(long)]
    search: Option<String>,
    /// Show the one-time welcome window (AI on this computer, which local runtime) and exit; does
    /// nothing when it has already been shown.
    #[arg(long)]
    first_run: bool,
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
