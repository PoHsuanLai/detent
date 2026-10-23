//! What the person's own styles have to say: the notes under "Use my styles" on the Appearance
//! page, and the sheet that lists them (customization design section 7.5). The lint that finds
//! them is quire's; this is only how they are counted and worded for the window.

use crate::words::{note_count_words, note_place_words};

/// Whose style file this is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleOwner {
    /// The one that styles the whole desktop.
    Desktop,
    /// An app's own, by the name of its folder.
    App(String),
}

impl StyleOwner {
    /// The file as the person finds it under their settings folder: `quire/style.css`.
    pub fn file_name(&self) -> String {
        match self {
            StyleOwner::Desktop => "quire/style.css".to_owned(),
            StyleOwner::App(folder) => format!("{folder}/style.css"),
        }
    }
}

/// One thing worth telling the person about a line of their styles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleNote {
    /// The 1-based line.
    pub line: u32,
    /// What it says, in plain words.
    pub words: String,
}

/// One style file and what it has to say.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleFile {
    /// Whose it is.
    pub owner: StyleOwner,
    /// Its notes, in the order of the lines.
    pub notes: Vec<StyleNote>,
}

/// One line of the sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRow {
    /// What the note says.
    pub words: String,
    /// Which file and line it is on.
    pub place: String,
}

/// The notes of every style file that could be read and has any. A file that could not be read is
/// not in it, and neither is one with nothing to say.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleNotes {
    files: Vec<StyleFile>,
}

impl StyleNotes {
    /// The notes of `files`, leaving out the files with none.
    pub fn new(files: Vec<StyleFile>) -> StyleNotes {
        StyleNotes {
            files: files
                .into_iter()
                .filter(|file| !file.notes.is_empty())
                .collect(),
        }
    }

    /// How many notes there are in all.
    pub fn total(&self) -> usize {
        self.files.iter().map(|file| file.notes.len()).sum()
    }

    /// `3 notes`, or `None` when there are none, so no line is drawn.
    pub fn summary(&self) -> Option<String> {
        match self.total() {
            0 => None,
            count => Some(note_count_words(count)),
        }
    }

    /// The sheet's lines: each file's notes in turn, each with its file and line.
    pub fn rows(&self) -> Vec<NoteRow> {
        self.files
            .iter()
            .flat_map(|file| {
                let name = file.owner.file_name();
                file.notes.iter().map(move |note| NoteRow {
                    words: note.words.clone(),
                    place: note_place_words(&name, note.line),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{NoteRow, StyleFile, StyleNote, StyleNotes, StyleOwner};

    fn file(owner: StyleOwner, notes: &[(u32, &str)]) -> StyleFile {
        StyleFile {
            owner,
            notes: notes
                .iter()
                .map(|(line, words)| StyleNote {
                    line: *line,
                    words: (*words).to_owned(),
                })
                .collect(),
        }
    }

    #[test]
    fn the_summary_counts_every_note_and_is_silent_with_none() {
        let app = |name: &str| StyleOwner::App(name.to_owned());
        // `(case, files, the summary)`.
        let cases: Vec<(&str, Vec<StyleFile>, Option<&str>)> = vec![
            ("no files", vec![], None),
            (
                "a file with nothing to say",
                vec![file(app("sill"), &[])],
                None,
            ),
            (
                "one note is singular",
                vec![file(StyleOwner::Desktop, &[(3, "Animates everything")])],
                Some("1 note"),
            ),
            (
                "two files add up",
                vec![
                    file(StyleOwner::Desktop, &[(1, "a"), (2, "b")]),
                    file(app("sill"), &[(9, "c")]),
                ],
                Some("3 notes"),
            ),
        ];
        for (case, files, want) in cases {
            assert_eq!(StyleNotes::new(files).summary().as_deref(), want, "{case}");
        }
    }

    #[test]
    fn the_sheet_names_the_file_and_line_of_each_note() {
        let notes = StyleNotes::new(vec![
            file(
                StyleOwner::Desktop,
                &[
                    (4, "Heavy blur may slow the desktop"),
                    (12, "This animation never stops"),
                ],
            ),
            file(StyleOwner::App("sill".to_owned()), &[]),
            file(
                StyleOwner::App("files".to_owned()),
                &[(2, "Could not be read")],
            ),
        ]);
        let row = |words: &str, place: &str| NoteRow {
            words: words.to_owned(),
            place: place.to_owned(),
        };
        assert_eq!(
            notes.rows(),
            [
                row("Heavy blur may slow the desktop", "quire/style.css, line 4"),
                row("This animation never stops", "quire/style.css, line 12"),
                row("Could not be read", "files/style.css, line 2"),
            ]
        );
    }
}
