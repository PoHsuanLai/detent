//! Driving the window the way a person does.
#![allow(dead_code)]

use ds::prelude::ShortcutKey;
use ds_harness::{Driver, Harness, Input, Query};
use std::time::Duration;

/// How long a click or a keystroke is given to reach the document and its handlers.
const SETTLE: Duration = Duration::from_millis(400);

/// Clicks the first element matching `selector`.
///
/// # Panics
/// If nothing matches: the failure carries the document, so the selector can be fixed.
pub fn click(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not on screen:\n{}", harness.html()));
    harness.send(Input::click(at));
    harness.advance(SETTLE);
}

/// Types `text` into the search field: focus it by a click, then one key per letter.
pub fn search_for(harness: &mut Harness, text: &str) {
    click(harness, ".ds-sidebar .ds-input");
    for letter in text.chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    harness.advance(SETTLE);
}

/// The page the window shows, by its slug.
pub fn page_of(harness: &Harness) -> Option<String> {
    harness.attr(".scroll", "data-page")
}

/// The name in the titlebar.
pub fn title_of(harness: &Harness) -> Option<String> {
    harness.text_of(".ds-titlebar-name")
}

/// The sidebar row `n` (from 1).
pub fn sidebar_row(n: usize) -> String {
    format!(".ds-list-item:nth-child({n}) .ds-row")
}
