//! How adding a picture ended, and the words for the ways it fails.

use std::path::PathBuf;

/// How asking for a picture and keeping it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddOutcome {
    /// The picture is kept: where it lies now, the value the key stores.
    Added(PathBuf),
    /// The person closed the chooser: nothing is said.
    Closed,
    /// The chosen file (by name) is not a kind of picture the desktop draws.
    NotAPicture(String),
    /// The chosen picture (by name) could not be copied among the added ones.
    CouldNotCopy(String),
    /// There is no window to choose a picture in.
    NoChooser,
}

impl AddOutcome {
    /// The sentence that tells the person why nothing was added; none for a picture added or a
    /// chooser the person closed.
    pub fn failure(&self) -> Option<String> {
        match self {
            AddOutcome::Added(_) | AddOutcome::Closed => None,
            AddOutcome::NotAPicture(name) => Some(format!(
                "\u{201c}{name}\u{201d} can\u{2019}t be used as a wallpaper. Choose a PNG, JPEG or WebP picture."
            )),
            AddOutcome::CouldNotCopy(name) => Some(format!(
                "\u{201c}{name}\u{201d} couldn\u{2019}t be added. Check that there is room in your home folder, then try again."
            )),
            AddOutcome::NoChooser => Some(
                "The window for choosing a picture couldn\u{2019}t be opened. Try again in a moment."
                    .to_owned(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AddOutcome;
    use std::path::PathBuf;

    #[test]
    fn only_a_refusal_is_told_and_it_names_the_file() {
        assert_eq!(AddOutcome::Added(PathBuf::from("/a.png")).failure(), None);
        assert_eq!(AddOutcome::Closed.failure(), None);
        let told = AddOutcome::NotAPicture("IMG_1.heic".to_owned()).failure();
        assert_eq!(
            told.as_deref(),
            Some(
                "\u{201c}IMG_1.heic\u{201d} can\u{2019}t be used as a wallpaper. Choose a PNG, JPEG or WebP picture."
            )
        );
        assert!(
            AddOutcome::CouldNotCopy("a.png".to_owned())
                .failure()
                .is_some()
        );
        assert!(AddOutcome::NoChooser.failure().is_some());
    }
}
