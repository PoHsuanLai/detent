//! How much a read gives back (audit DETENT-10). The manifest promises the router a size for
//! each value Settings returns; a key can hold anything the person or a program wrote, so the
//! provider cuts what it returns to what the manifest declares and does not rely on the router
//! to do it. A test holds the numbers equal to the manifest's.

/// `settings.key.read`: the value as text, `max = 500`, one line.
pub(crate) const KEY_VALUE_CHARS: usize = 500;

/// `settings.accounts.list`: one line an account, `max = 2000`.
pub(crate) const ACCOUNTS_CHARS: usize = 2000;

/// `text` cut to at most `max` characters, the last of them an ellipsis when it was cut.
fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut kept: String = text.chars().take(max.saturating_sub(1)).collect();
    kept.push('\u{2026}');
    kept
}

/// `text` as one line of at most `max` characters: line breaks become spaces.
pub(crate) fn one_line(text: &str, max: usize) -> String {
    cut(&text.replace(['\n', '\r'], " "), max)
}

/// `text` with its line breaks, cut to at most `max` characters.
pub(crate) fn many_lines(text: &str, max: usize) -> String {
    cut(text, max)
}

#[cfg(test)]
mod tests {
    use super::{ACCOUNTS_CHARS, KEY_VALUE_CHARS, many_lines, one_line};
    use crate::manifest::manifest;
    use docket_core::{Lines, ParamType, ResultShape};

    /// The text size and line rule the manifest declares for `action`'s result.
    fn declared(action: &str) -> (usize, Lines) {
        let valid = manifest().expect("a valid manifest");
        let found = valid
            .manifest()
            .actions
            .iter()
            .find(|a| a.name.as_str() == action)
            .unwrap_or_else(|| panic!("{action} is declared"));
        match &found.result {
            ResultShape::Value {
                ty: ParamType::Text { max, lines },
                ..
            } => (usize::try_from(max.0).expect("a size"), *lines),
            other => panic!("{action} returns {other:?}"),
        }
    }

    #[test]
    fn the_limits_are_what_the_manifest_declares() {
        assert_eq!(declared("settings.key.read"), (KEY_VALUE_CHARS, Lines::One));
        assert_eq!(
            declared("settings.accounts.list"),
            (ACCOUNTS_CHARS, Lines::Many)
        );
    }

    #[test]
    fn a_long_value_is_cut_and_a_short_one_is_not() {
        assert_eq!(one_line("on", 500), "on");
        let cut = one_line(&"x".repeat(5000), 500);
        assert_eq!(cut.chars().count(), 500);
        assert!(cut.ends_with('\u{2026}'));
    }

    #[test]
    fn a_value_with_line_breaks_is_one_line() {
        assert_eq!(one_line("a\nb\r\nc", 500), "a b  c");
        assert_eq!(many_lines("a\nb", 500), "a\nb");
    }

    #[test]
    fn a_cut_never_splits_a_character() {
        let wide = "\u{3042}".repeat(900);
        assert_eq!(many_lines(&wide, 2000), wide);
        assert_eq!(one_line(&wide, 500).chars().count(), 500);
    }
}
