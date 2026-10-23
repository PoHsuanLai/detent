//! Turning a schema's stored words into labels.

use ds_settings::schema::WordLabels;

/// What a person reads for `word`: the schema's own label when it gave one, else the word
/// humanized.
pub fn word_label(labels: &WordLabels, word: &str) -> String {
    labels
        .of(word)
        .map_or_else(|| humanize(word), str::to_owned)
}

/// `when_scrolling` as `When scrolling`: underscores to spaces, the first letter upper case.
pub fn humanize(word: &str) -> String {
    let spaced = word.replace('_', " ");
    let mut letters = spaced.chars();
    match letters.next() {
        Some(first) => first.to_uppercase().chain(letters).collect(),
        None => String::new(),
    }
}

/// The sheet's title, and the label of the line that opens it.
pub const STYLE_NOTES_TITLE: &str = "Notes on your styles";

/// The sentence over the notes in the sheet.
pub const STYLE_NOTES_LEAD: &str = "These do not stop your styles from working. They point at things that may slow the desktop, change in an update or have no effect.";

/// `3 notes`, or `1 note`: how many things the person's styles have to say.
pub fn note_count_words(count: usize) -> String {
    match count {
        1 => "1 note".to_owned(),
        many => format!("{many} notes"),
    }
}

/// Where a note is: `quire/style.css, line 4`.
pub fn note_place_words(file: &str, line: u32) -> String {
    format!("{file}, line {line}")
}

#[cfg(test)]
mod tests {
    use super::word_label;
    use ds_settings::schema::WordLabels;

    #[test]
    fn a_stored_word_reads_as_a_label() {
        let labels =
            WordLabels([("workspace_prev".to_owned(), "Previous workspace".to_owned())].into());
        // `(stored word, what is read)`: the schema's label wins, else the word humanized.
        const CASES: &[(&str, &str)] = &[
            ("when_scrolling", "When scrolling"),
            ("on", "On"),
            ("top_right", "Top right"),
            ("", ""),
            ("h24", "H24"),
            ("workspace_prev", "Previous workspace"),
            ("smart_zoom", "Smart zoom"),
        ];
        for (word, want) in CASES {
            assert_eq!(word_label(&labels, word), *want, "{word:?}");
        }
    }
}
