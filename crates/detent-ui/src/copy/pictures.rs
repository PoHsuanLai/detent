//! The picture chooser's words: why nothing was added.

use detent_model::AddOutcome;

/// The sentence that tells the person why nothing was added; none for a picture added or a
/// chooser the person closed.
pub(crate) fn add_failure(outcome: &AddOutcome) -> Option<String> {
    match outcome {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn only_a_refusal_is_told_and_it_names_the_file() {
        let cases = [
            (AddOutcome::Added(PathBuf::from("/a.png")), None),
            (AddOutcome::Closed, None),
            (
                AddOutcome::NotAPicture("IMG_1.heic".to_owned()),
                Some(
                    "\u{201c}IMG_1.heic\u{201d} can\u{2019}t be used as a wallpaper. Choose a PNG, JPEG or WebP picture.",
                ),
            ),
            (
                AddOutcome::CouldNotCopy("a.png".to_owned()),
                Some(
                    "\u{201c}a.png\u{201d} couldn\u{2019}t be added. Check that there is room in your home folder, then try again.",
                ),
            ),
            (
                AddOutcome::NoChooser,
                Some(
                    "The window for choosing a picture couldn\u{2019}t be opened. Try again in a moment.",
                ),
            ),
        ];
        for (outcome, want) in cases {
            assert_eq!(add_failure(&outcome).as_deref(), want);
        }
    }
}
