//! The Spaces page's list over a registry that answers from memory: Remove asks on the sheet
//! first, the choice reaches the registry as the person made it, and Cancel sends nothing. The
//! real link's order (memory is told before the registry is asked, and a delete choice with no
//! memory running goes no further) is tested over a private bus in detent-system.

use crate::support::Sandbox;
use crate::support::drive::click;
use detent_model::{PageSlug, SpaceKey, SpaceRow};
use detent_system::spaces::FakeSpaces;
use detent_ui::Opening;
use ds_harness::Harness;
use std::sync::Arc;
use std::time::Duration;

const TRIP: &str = "[aria-label=\"Remove Trip\"]";
const SHEET: &str = ".space-sheet";
const REMOVE: &str = ".space-sheet .ds-button[data-answers=\"return\"]";
const CANCEL: &str = ".space-sheet .ds-button[data-answers=\"escape\"]";
const FLIP: &str = ".space-sheet .ds-button[data-variant=\"inline\"]";
const DELETE_HISTORY: &str = ".space-sheet [role=\"radio\"]:last-child";

fn spaces() -> Opening {
    Opening {
        page: Some(PageSlug("spaces".to_owned())),
        query: String::new(),
    }
}

/// Lets the page hear what the registry said: the follow loop runs on the harness's clock.
fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    for _ in 0..200 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

struct World {
    fake: FakeSpaces,
    harness: Harness,
}

fn row(id: &str, name: &str) -> SpaceRow {
    SpaceRow {
        id: SpaceKey(id.to_owned()),
        name: name.to_owned(),
    }
}

fn world(name: &str) -> World {
    let fake = FakeSpaces::holding(vec![row("space-1", "Trip"), row("space-2", "Work")]);
    let sandbox = Sandbox::new(name).with_spaces(Arc::new(fake.clone()));
    let mut harness = sandbox.harness(spaces());
    settle(&mut harness, "the list", |h| h.count(TRIP) == 1);
    World { fake, harness }
}

/// What the registry was asked to remove, as `space <choice>` in order.
fn asked(w: &World) -> Vec<String> {
    w.fake
        .removals()
        .into_iter()
        .map(|(space, removal)| format!("{} {removal:?}", space.0))
        .collect()
}

fn open_sheet(w: &mut World) {
    click(&mut w.harness, TRIP);
    settle(&mut w.harness, "the sheet", |h| h.count(SHEET) == 1);
}

#[test]
fn the_sheet_opens_on_the_keeping_choices_and_says_its_words() {
    let mut w = world("spaces-sheet");
    assert!(asked(&w).is_empty());
    open_sheet(&mut w);
    let html = w.harness.html();
    for text in [
        "Remove \u{201c}Trip\u{201d}?",
        "Its memories move to the apps that saved them.",
        "Keep its history",
        "Delete its history",
        "History is what happened in this Space, like files you saved and messages you sent.",
        "Delete its memories instead",
    ] {
        assert!(html.contains(text), "{text}");
    }
    assert_eq!(
        w.harness
            .attr(".space-sheet [role=\"radio\"]", "aria-checked")
            .as_deref(),
        Some("true"),
        "the first choice is selected"
    );
    assert_eq!(
        w.harness.attr(REMOVE, "data-role").as_deref(),
        Some("normal"),
        "nothing is lost, so Remove is not destructive"
    );
}

#[test]
fn cancel_sends_nothing_and_the_space_stays() {
    let mut w = world("spaces-cancel");
    open_sheet(&mut w);
    click(&mut w.harness, CANCEL);
    settle(&mut w.harness, "the sheet to go", |h| h.count(SHEET) == 0);
    assert!(asked(&w).is_empty());
    assert_eq!(w.harness.count(TRIP), 1);
}

#[test]
fn remove_asks_for_the_keeping_choices_and_the_row_goes() {
    let mut w = world("spaces-remove");
    open_sheet(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(
        asked(&w),
        ["space-1 Removal { memories: MoveToApps, history: Keep }".to_owned()]
    );
    assert_eq!(w.harness.count("[aria-label=\"Remove Work\"]"), 1);
}

#[test]
fn the_link_flips_the_body_and_its_label_and_makes_remove_destructive() {
    let mut w = world("spaces-flip");
    open_sheet(&mut w);
    click(&mut w.harness, FLIP);
    settle(&mut w.harness, "the flipped body", |h| {
        h.html().contains("Its memories will be deleted.")
    });
    let html = w.harness.html();
    assert!(html.contains("Move its memories instead"));
    assert!(!html.contains("Delete its memories instead"));
    assert_eq!(
        w.harness.attr(REMOVE, "data-role").as_deref(),
        Some("destructive")
    );
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(
        asked(&w),
        ["space-1 Removal { memories: Delete, history: Keep }".to_owned()]
    );
}

#[test]
fn deleting_the_history_alone_makes_remove_destructive_and_is_sent() {
    let mut w = world("spaces-history");
    open_sheet(&mut w);
    click(&mut w.harness, DELETE_HISTORY);
    settle(&mut w.harness, "the choice", |h| {
        h.attr(REMOVE, "data-role").as_deref() == Some("destructive")
    });
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the row to go", |h| h.count(TRIP) == 0);
    assert_eq!(
        asked(&w),
        ["space-1 Removal { memories: MoveToApps, history: Delete }".to_owned()]
    );
}

#[test]
fn a_refused_removal_says_why_in_a_toast_and_the_space_stays() {
    let mut w = world("spaces-refused");
    w.fake.refuses("Nothing could be removed just now.");
    open_sheet(&mut w);
    click(&mut w.harness, REMOVE);
    settle(&mut w.harness, "the toast", |h| h.count(".ds-toast") == 1);
    let toast = w.harness.text_of(".ds-toast").unwrap_or_default();
    assert!(
        toast.contains("Nothing could be removed just now."),
        "{toast}"
    );
    assert_eq!(w.harness.count(TRIP), 1, "the Space stays");
}
