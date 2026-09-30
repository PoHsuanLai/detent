//! Turning a schema's stored words into labels.

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
    use super::humanize;

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
}
