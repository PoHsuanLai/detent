//! The assistant's model: its page in the sidebar and its first-run step, over a fake inferd
//! module in memory. Nothing is chosen until the person chooses, and an unset model shows as such.

use crate::support::Sandbox;
use crate::support::drive::{click, scroll_into_view, sidebar_page};
use crate::support::inferd::{Inferd, Policy};
use detent_model::{ASSISTANT_MODEL_SLUG, ASSISTANT_SETUP_SLUG, Notice, PageSlug};
use detent_ui::Opening;
use ds_harness::{Harness, Input};
use std::time::Duration;

mod first_run;
mod modes;
mod page;

const ROW: &str = "ai.model.text.balanced";
const BUTTON: &str = ".assistant-model .ds-popup .ds-button";
const SETUP_BUTTON: &str = ".assistant-setup .ds-popup .ds-button";
const CONTINUE: &str = ".assistant-continue";
const SKIP: &str = ".assistant-skip";

fn opening(slug: &str) -> Opening {
    Opening {
        page: Some(PageSlug(slug.to_owned())),
        query: String::new(),
    }
}

fn settle(harness: &mut Harness, what: &str, done: impl Fn(&Harness) -> bool) {
    // The fake daemons answer on real threads; a loaded machine running every test at once can
    // take several seconds, so the budget is generous (15 s) and costs nothing when it settles.
    for _ in 0..750 {
        if done(harness) {
            return;
        }
        harness.advance(Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{what} never happened:\n{}", harness.html());
}

/// The row's button text, once the picker is there.
fn shown(harness: &Harness, button: &str) -> Option<String> {
    harness
        .text_of(&format!("{button} .ds-button-label"))
        .map(|text| text.trim().to_owned())
}

/// Opens the row's menu and returns the number of items marked chosen.
fn open_and_count_marked(harness: &mut Harness, button: &str) -> usize {
    scroll_into_view(harness, button);
    click(harness, button);
    assert_eq!(
        harness.attr(button, "aria-expanded").as_deref(),
        Some("true"),
        "the click opened the menu"
    );
    harness.count(".ds-menu-item[aria-checked=\"true\"]")
}

/// Clicks the menu item `nth` (from 1), the way a person points at it first.
fn pick(harness: &mut Harness, nth: usize) {
    let label = format!(".ds-menu-item:nth-child({nth}) .ds-menu-label");
    let at = harness.centre(&label).expect("the menu is open");
    harness.send(Input::pointer_move(at));
    click(harness, &label);
}

/// Inferd's module serving slots in memory, the sandbox looking at it, and the page's harness.
struct World {
    fake: Inferd,
    harness: Harness,
}

fn world(name: &str, slug: &str, fake: Inferd) -> World {
    let harness = Sandbox::new(name)
        .with_modules(fake.hosted())
        .harness(opening(slug));
    World { fake, harness }
}

fn ready(world: &mut World) {
    settle(&mut world.harness, "the picker", |h| {
        h.attr(".picker", "data-picker").as_deref() == Some("ready")
    });
}
