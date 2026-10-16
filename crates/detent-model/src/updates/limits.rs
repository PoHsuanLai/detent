//! What the update service may make the page hold (audit DETENT-06). Its words are shown as they
//! come, and a service that is not the real one (or a real one with a bug) must not be able to
//! fill the window: a version is a short run of version characters, the notes are a few screens,
//! the problem a sentence.

/// The longest version the page shows: far beyond any real one (`0.12.3-rc.4+build.5`).
const MAX_VERSION_CHARS: usize = 64;

/// The most of the release notes the page keeps: a few screens of text.
const MAX_NOTES_BYTES: usize = 16 * 1024;

/// The most lines of the release notes the page draws, one label each.
const MAX_NOTES_LINES: usize = 200;

/// The longest problem sentence the page shows.
const MAX_PROBLEM_CHARS: usize = 300;

/// Whether `c` may appear in a version: digits, letters and the marks versions use.
fn is_version_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '~' | '-' | '_')
}

/// `text` as a version, when it is one: non-empty, short, and only version characters. A service
/// whose version is anything else is not making sense.
pub(super) fn version_of(text: &str) -> Option<String> {
    let fits = !text.is_empty()
        && text.chars().count() <= MAX_VERSION_CHARS
        && text.chars().all(is_version_char);
    fits.then(|| text.to_owned())
}

/// The first `MAX_NOTES_LINES` lines of `notes`, at most `MAX_NOTES_BYTES` of them, cut on a
/// character boundary (a longer line is cut where the room ends).
pub(super) fn notes_of(notes: &str) -> String {
    let mut kept = String::new();
    for (at, line) in notes.lines().take(MAX_NOTES_LINES).enumerate() {
        if at > 0 {
            if kept.len() >= MAX_NOTES_BYTES {
                return kept;
            }
            kept.push('\n');
        }
        for c in line.chars() {
            if kept.len() + c.len_utf8() > MAX_NOTES_BYTES {
                return kept;
            }
            kept.push(c);
        }
    }
    kept
}

/// `text` as one short sentence: control characters become spaces and it ends at
/// `MAX_PROBLEM_CHARS` with an ellipsis.
pub(super) fn problem_of(text: &str) -> String {
    let flat: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let mut shown: String = flat.chars().take(MAX_PROBLEM_CHARS).collect();
    if flat.chars().count() > MAX_PROBLEM_CHARS {
        shown.push('\u{2026}');
    }
    shown
}
