//! One section of pictures: a heading over a grid of tiles that shows every row at once, so the
//! page scrolls and the grid does not.

use super::Smalls;
use detent_model::{PictureTile, current_place};
use dioxus::prelude::*;
use ds::components::content::icon_source::IconSource;
use ds::components::lists::collection_view::{
    CollectionItem, CollectionSize, CollectionView, Flow, Thumbnail,
};
use ds::prelude::*;

/// How many tiles make a row: four of the large size's tiles fit the settings window's content
/// at its first width (a tile asks for at least `CollectionSize::Large.tile()`).
const COLUMNS: u8 = 4;

/// The size every tile is drawn at.
const SIZE: CollectionSize = CollectionSize::Large;

/// The tile's picture: its small picture when that is made, else a picture glyph.
fn thumbnail_of(tile: &PictureTile, smalls: &Smalls) -> Thumbnail {
    smalls
        .get(&tile.preview)
        .and_then(|small| ImageSource::file(small).ok())
        .map_or(
            Thumbnail::Icon(IconSource::Glyph(Icon::Image)),
            Thumbnail::Image,
        )
}

/// The height of all of the grid's rows, so the grid never needs a scroll area of its own.
fn height_of(tiles: usize) -> f32 {
    tiles.div_ceil(usize::from(COLUMNS)) as f32 * SIZE.pitch().0
}

/// The section called `title`: `tiles` as a grid with the current one selected. A pick or an
/// open (a double click) of a tile is sent to `onpick` as the tile's path.
pub(super) fn pictures_grid(
    title: &str,
    tiles: &[PictureTile],
    smalls: &Smalls,
    onpick: EventHandler<String>,
) -> Element {
    let paths: Vec<String> = tiles
        .iter()
        .map(|tile| tile.path.to_string_lossy().into_owned())
        .collect();
    let items: Vec<CollectionItem<String>> = tiles
        .iter()
        .zip(&paths)
        .map(|(tile, path)| {
            CollectionItem::new(path.clone(), tile.name.clone(), thumbnail_of(tile, smalls))
        })
        .collect();
    let height = height_of(tiles.len());
    let on_select = paths.clone();
    let on_open = paths;
    rsx! {
        div { class: "key-pictures-section",
            SectionHeader { title: title.to_owned() }
            div { class: "key-pictures-grid", style: "height:{height}px",
                CollectionView::<String> {
                    label: title.to_owned(),
                    items,
                    size: SIZE,
                    flow: Flow::Columns(COLUMNS),
                    cursor: current_place(tiles),
                    onselect: move |at: usize| {
                        if let Some(path) = on_select.get(at) {
                            onpick.call(path.clone());
                        }
                    },
                    onopen: move |at: usize| {
                        if let Some(path) = on_open.get(at) {
                            onpick.call(path.clone());
                        }
                    },
                }
            }
        }
    }
}
