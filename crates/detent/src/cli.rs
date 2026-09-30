//! The command line: where the window opens.

use clap::Parser;
use detent_model::PageSlug;
use detent_ui::Opening;

/// The desktop's Settings app.
#[derive(Debug, Parser)]
#[command(name = "detent", version)]
pub struct Cli {
    /// Open on this page (`appearance`, `dock`, `mouse_and_gestures`, ...).
    #[arg(long)]
    page: Option<String>,
    /// Open with this in the search field.
    #[arg(long)]
    search: Option<String>,
}

impl Cli {
    /// Where the window opens.
    pub fn opening(self) -> Opening {
        Opening {
            page: self.page.map(PageSlug),
            query: self.search.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;
    use detent_model::PageSlug;

    #[test]
    fn the_flags_say_where_the_window_opens() {
        let cli = Cli::try_parse_from(["detent", "--page", "dock", "--search", "magnification"])
            .expect("valid flags");
        let opening = cli.opening();
        assert_eq!(opening.page, Some(PageSlug("dock".to_owned())));
        assert_eq!(opening.query, "magnification");
    }

    #[test]
    fn no_flags_open_on_the_first_page() {
        let opening = Cli::try_parse_from(["detent"]).expect("no flags").opening();
        assert_eq!(opening.page, None);
        assert_eq!(opening.query, "");
    }
}
