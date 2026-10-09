//! Driving the window the way a person does.
#![allow(dead_code)]

use ds::prelude::{Point, Px, ShortcutKey};
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

/// Scrolls the page with the wheel until the element matching `selector` is inside the window.
///
/// # Panics
/// If the element is missing, or still below the window after the page has been scrolled as far
/// as it goes.
pub fn scroll_into_view(harness: &mut Harness, selector: &str) {
    let content = Point {
        x: Px(600.0),
        y: Px(300.0),
    };
    let window = f64::from(super::VIEW.height);
    for _ in 0..40 {
        let rect = harness
            .rect(selector)
            .unwrap_or_else(|| panic!("{selector} is not on the page:\n{}", harness.html()));
        if f64::from(rect.origin.y.0 + rect.size.height.0) < window - 8.0 {
            return;
        }
        harness.send(Input::wheel(content, Px(0.0), Px(-200.0)));
        harness.advance(Duration::from_millis(100));
    }
    panic!("{selector} never came into view");
}
