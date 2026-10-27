//! What the person's own styles have to say: the notes under "Use my styles" on the Appearance
//! page, and the sheet that lists them (customization design section 7.5). The lint that finds
//! them is quire's; this is only how they are counted and placed for the window.

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
    /// What it says, in the plain words quire's lint gives it.
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
    /// The file it is in, as the person finds it under their settings folder.
    pub file: String,
    /// The 1-based line it is on.
    pub line: u32,
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

    /// The sheet's lines: each file's notes in turn, each with its file and line.
    pub fn rows(&self) -> Vec<NoteRow> {
        self.files
            .iter()
            .flat_map(|file| {
                let name = file.owner.file_name();
                file.notes.iter().map(move |note| NoteRow {
                    words: note.words.clone(),
                    file: name.clone(),
                    line: note.line,
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
    fn the_total_counts_every_note_of_every_file_that_has_any() {
        let app = |name: &str| StyleOwner::App(name.to_owned());
        // `(case, files, the total)`.
        let cases: Vec<(&str, Vec<StyleFile>, usize)> = vec![
            ("no files", vec![], 0),
            (
                "a file with nothing to say",
                vec![file(app("sill"), &[])],
                0,
            ),
            (
                "one note",
                vec![file(StyleOwner::Desktop, &[(3, "Animates everything")])],
                1,
            ),
            (
                "two files add up",
                vec![
                    file(StyleOwner::Desktop, &[(1, "a"), (2, "b")]),
                    file(app("sill"), &[(9, "c")]),
                ],
                3,
            ),
        ];
        for (case, files, want) in cases {
            assert_eq!(StyleNotes::new(files).total(), want, "{case}");
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
        let row = |words: &str, file: &str, line: u32| NoteRow {
            words: words.to_owned(),
            file: file.to_owned(),
            line,
        };
        assert_eq!(
            notes.rows(),
            [
                row("Heavy blur may slow the desktop", "quire/style.css", 4),
                row("This animation never stops", "quire/style.css", 12),
                row("Could not be read", "files/style.css", 2),
            ]
        );
    }
}
