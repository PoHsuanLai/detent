//! The Intelligence page: its sections in design/22's order, inferd's model picker over a fake
//! settings module in memory, and the page without inferd.

use crate::support::Sandbox;
use crate::support::drive::{click, scroll_into_view};
use crate::support::inferd::{Inferd, Policy};
use detent_client::ReasonFeed;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds_harness::{Harness, Input};
use porter_core::{AccountId, ModelId};
use porter_infer::{Declined, DeclinedBecause, InferEvent, ModelRef, Why};
use std::time::Duration;

mod absent;
mod picker;
mod sections;

fn intelligence() -> Opening {
    Opening {
        page: Some(PageSlug("intelligence".to_owned())),
        query: String::new(),
    }
}

/// A one-key schema of a program that has none of its own in the fixtures yet, written the way
/// design/22 section 3 names its keys.
fn stand_in(app: &str, file: &str, path: &str, section: &str) -> String {
    format!(
        "app = \"{app}\"\nfile = \"{file}\"\nversion = 1\n\n[[key]]\npath = \"{path}\"\n\
         default = \"on\"\nlabel = \"{path}\"\nhelp = \"\"\nsection = \"{section}\"\n\
         exposure = \"basic\"\n\n[key.kind]\nkind = \"toggle\"\n\n[key.kind.v]\n\
         variants = [\"on\", \"off\"]\n\n[key.page]\nkind = \"intelligence\"\n"
    )
}

/// Runs the page's async work: advance the document until `done` holds.
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

/// Positions of each of `texts` in `html`, each searched for after the one before.
fn in_order(html: &str, texts: &[&str]) -> Vec<usize> {
    let mut from = 0;
    texts
        .iter()
        .map(|text| {
            let at = html[from..]
                .find(&format!(">{text}<"))
                .unwrap_or_else(|| panic!("{text:?} is not on the page after byte {from}"));
            from += at;
            from
        })
        .collect()
}

fn model(name: &str) -> ModelRef {
    ModelRef {
        account: AccountId::parse("local").expect("account"),
        model: ModelId::parse(name).expect("model"),
    }
}
