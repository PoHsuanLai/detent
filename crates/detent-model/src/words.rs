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

#[cfg(test)]
mod tests {
    use super::{humanize, word_label};
    use ds_settings::schema::WordLabels;

    #[test]
    fn a_stored_word_reads_as_a_label() {
        const CASES: &[(&str, &str)] = &[
            ("when_scrolling", "When scrolling"),
            ("on", "On"),
            ("top_right", "Top right"),
            ("", ""),
            ("h24", "H24"),
        ];
        for (word, want) in CASES {
            assert_eq!(humanize(word), *want, "{word:?}");
        }
    }

    #[test]
    fn a_schema_label_wins_over_the_humanized_word() {
        let labels =
            WordLabels([("workspace_prev".to_owned(), "Previous workspace".to_owned())].into());
        assert_eq!(word_label(&labels, "workspace_prev"), "Previous workspace");
        assert_eq!(word_label(&labels, "smart_zoom"), "Smart zoom");
    }
}
