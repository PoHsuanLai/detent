//! A picture: the pictures on offer in rows of thumbnails with the current one checked, and the
//! "Add Picture..." button under them.

use crate::env::Env;
use detent_model::{PictureMark, PictureTile, picture_tiles};
use dioxus::prelude::*;
use ds::components::controls::radio_group::Arrangement;
use ds::prelude::*;
use ds::style::icon::render::IconPx;
use ds::style::icon::url::IconUrl;
use std::path::PathBuf;

/// How many thumbnails make a row.
const PER_ROW: usize = 4;

/// A thumbnail's side in logical pixels.
const THUMB: u8 = 72;

/// The choice a tile makes: its path, its name beneath a thumbnail of it.
fn choice_of(tile: &PictureTile) -> Option<Choice<String>> {
    let url = IconUrl::file(&tile.path).ok()?;
    let size = IconSize::Px(IconPx(THUMB));
    Some(
        Choice::new(tile.path.to_string_lossy().into_owned(), tile.name.clone())
            .with_icon(IconSource::Image(ExternalIcon { url, size })),
    )
}

/// The path of the tile marked current, or empty when none is.
fn current_of(tiles: &[PictureTile]) -> String {
    tiles
        .iter()
        .find(|tile| tile.mark == PictureMark::Current)
        .map(|tile| tile.path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The rows of pictures and the button that adds one; a pick or an added picture is sent to
/// `onchange` as the path to store.
#[component]
pub fn PictureControl(
    label: String,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let pictures = use_context::<Env>().pictures;
    let mut found = use_signal(Vec::<PathBuf>::new);
    use_future({
        let pictures = pictures.clone();
        move || {
            let pictures = pictures.clone();
            async move { found.set(pictures.list().await) }
        }
    });
    let stored = value.as_str().unwrap_or_default().to_owned();
    let tiles = picture_tiles(&found.read(), &stored);
    let current = current_of(&tiles);
    let rows: Vec<Vec<Choice<String>>> = tiles
        .chunks(PER_ROW)
        .map(|row| row.iter().filter_map(choice_of).collect())
        .collect();
    rsx! {
        div { class: "key-pictures",
            for (index , choices) in rows.into_iter().enumerate() {
                RadioGroup::<String> {
                    key: "{index}",
                    label: label.clone(),
                    choices,
                    value: current.clone(),
                    arrangement: Arrangement::Row,
                    onchange: move |path: String| onchange.call(toml::Value::String(path)),
                }
            }
            Button {
                label: "Add Picture\u{2026}",
                onclick: move |_| {
                    let pictures = pictures.clone();
                    spawn(async move {
                        if let Some(path) = pictures.add().await {
                            onchange.call(toml::Value::String(path.to_string_lossy().into_owned()));
                            found.set(pictures.list().await);
                        }
                    });
                },
            }
        }
    }
}
