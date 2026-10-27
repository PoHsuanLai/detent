//! The top row of the Wallpaper page: the picture on the desktop, larger, with its name, and the
//! button that adds a picture; under it, why the last add gave nothing, when it did not.

use super::Smalls;
use detent_model::PictureTile;
use dioxus::prelude::*;
use ds::components::content::icon_source::IconSource;
use ds::components::content::icon_view::IconView;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::prelude::*;
use ds::style::icon::render::{IconPx, IconSize};

/// What the row says when no picture is stored.
const NO_PICTURE: &str = crate::copy::pictures::NO_PICTURE_CHOSEN;

/// The side of the glyph shown while the current picture's small picture is not made.
const GLYPH: u8 = 32;

/// The current picture, drawn from its small picture, or a picture glyph while that is not made.
fn current_preview(current: Option<&PictureTile>, smalls: &Smalls) -> Element {
    let small = current
        .and_then(|tile| smalls.get(&tile.preview))
        .and_then(|small| ImageSource::file(small).ok());
    match small {
        Some(source) => rsx! {
            img { class: "key-pictures-image", alt: "", src: "{source.0}", draggable: "false" }
        },
        None => rsx! {
            IconView {
                source: IconSource::Glyph(Icon::Image),
                size: IconSize::Px(IconPx(GLYPH)),
            }
        },
    }
}

/// The row for `current` (none: no picture is stored), with the button that calls `onadd`, and
/// `told` under it when the last try to add a picture came to nothing.
pub(super) fn picture_header(
    current: Option<&PictureTile>,
    smalls: &Smalls,
    told: Option<String>,
    onadd: EventHandler<()>,
) -> Element {
    let name = current.map_or_else(|| NO_PICTURE.to_owned(), |tile| tile.name.clone());
    rsx! {
        div { class: "key-pictures-head",
            div { class: "key-pictures-current", {current_preview(current, smalls)} }
            div { class: "key-pictures-title",
                Label { text: name, role: LabelRole::Primary }
            }
            Button {
                label: crate::copy::pictures::ADD_PICTURE_ELLIPSIS,
                onclick: move |_| onadd.call(()),
            }
        }
        if let Some(words) = told {
            div { class: "key-pictures-told",
                Label { text: words, role: LabelRole::Secondary, style: LabelStyle::Footnote }
            }
        }
    }
}
