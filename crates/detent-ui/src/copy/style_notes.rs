//! The notes on the person's own styles: the line that counts them and the sheet that lists them.

use detent_model::NoteRow;

/// The sheet's title, and the label of the line that opens it.
pub(crate) const STYLE_NOTES_TITLE: &str = "Notes on your styles";

/// The sentence over the notes in the sheet.
pub(crate) const STYLE_NOTES_LEAD: &str = "These do not stop your styles from working. They point at things that may slow the desktop, change in an update or have no effect.";

/// `3 notes`, or `1 note`: how many things the person's styles have to say.
pub(crate) fn notes_summary(count: usize) -> String {
    match count {
        1 => "1 note".to_owned(),
        many => format!("{many} notes"),
    }
}

/// Where a note is: `quire/style.css, line 4`.
pub(crate) fn note_place(row: &NoteRow) -> String {
    format!("{}, line {}", row.file, row.line)
}

/// The words "Show".
pub(crate) const SHOW: &str = "Show";

/// The words "Show {STYLE_NOTES_TITLE}".
pub(crate) fn show(style_notes_title: impl std::fmt::Display) -> String {
    format!("Show {style_notes_title}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_summary_counts_the_notes() {
        let cases = [(1, "1 note"), (2, "2 notes"), (3, "3 notes")];
        for (count, want) in cases {
            assert_eq!(notes_summary(count), want);
        }
    }

    #[test]
    fn a_note_is_placed_by_file_and_line() {
        let row = NoteRow {
            words: "Heavy blur may slow the desktop".to_owned(),
            file: "quire/style.css".to_owned(),
            line: 4,
        };
        assert_eq!(note_place(&row), "quire/style.css, line 4");
    }
}
