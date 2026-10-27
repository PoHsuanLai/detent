//! The head of an overview page: its tile, its name and one sentence on what it is for.

use crate::copy::heroes;
use crate::icons::tile_of;
use detent_model::{Hero, PageKind};
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::forms::icon_tile::tile;
use ds::prelude::*;

/// The hero of the page `title` of `kind`.
#[component]
pub(crate) fn HeroView(kind: PageKind, hero: Hero, title: String) -> Element {
    rsx! {
        div { class: "hero", "data-hero": "{title}",
            div { class: "hero-tile", {tile(tile_of(kind))} }
            Label { text: title.clone(), role: LabelRole::Primary, style: LabelStyle::Title }
            Label { text: heroes::line(hero), role: LabelRole::Secondary }
        }
    }
}
