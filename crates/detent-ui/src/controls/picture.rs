//! A picture: the pictures on offer in rows of thumbnails with the current one checked, and the
//! "Add Picture..." button under them.

use crate::Pictures;
use crate::env::Env;
use detent_model::{Picture, PictureMark, PictureTile, picture_tiles};
use dioxus::prelude::*;
use ds::components::controls::radio_group::Arrangement;
use ds::prelude::*;
use ds::style::icon::render::IconPx;
use ds::style::icon::url::IconUrl;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// How many thumbnails make a row.
const PER_ROW: usize = 4;

/// A thumbnail's side in logical pixels.
const THUMB: u8 = 72;

/// The small pictures made so far, by the file each is made from.
type Smalls = HashMap<PathBuf, PathBuf>;

/// The choice a tile makes: its path, its name beneath a small picture of it. A tile whose small
/// picture is not made yet, or cannot be made, is a plain tile with its name.
fn choice_of(tile: &PictureTile, smalls: &Smalls) -> Choice<String> {
    let choice = Choice::new(tile.path.to_string_lossy().into_owned(), tile.name.clone());
    let small = smalls
        .get(&tile.preview)
        .and_then(|small| IconUrl::file(small).ok());
    match small {
        Some(url) => {
            let size = IconSize::Px(IconPx(THUMB));
            choice.with_icon(IconSource::Image(ExternalIcon { url, size }))
        }
        None => choice,
    }
}

/// Makes the small picture of each of `previews` not in `smalls` yet, one after the other so the
/// first tiles fill in first, and puts each in `smalls` as it is made.
async fn make_smalls(
    pictures: Arc<dyn Pictures>,
    previews: Vec<PathBuf>,
    mut smalls: Signal<Smalls>,
) {
    for preview in previews {
        if smalls.peek().contains_key(&preview) {
            continue;
        }
        if let Some(small) = pictures.thumbnail(preview.clone()).await {
            smalls.write().insert(preview, small);
        }
    }
}

/// The path of the tile marked current, or empty when none is.
fn current_of(tiles: &[PictureTile]) -> String {
    tiles
        .iter()
        .find(|tile| tile.mark == PictureMark::Current)
        .map(|tile| tile.path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The files the tiles' small pictures are made from, in the order of the tiles.
fn previews_of(tiles: &[PictureTile]) -> Vec<PathBuf> {
    tiles.iter().map(|tile| tile.preview.clone()).collect()
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
    let mut found = use_signal(Vec::<Picture>::new);
    let smalls = use_signal(Smalls::new);
    let stored = value.as_str().unwrap_or_default().to_owned();
    use_future({
        let pictures = pictures.clone();
        let stored = stored.clone();
        move || {
            let pictures = pictures.clone();
            let stored = stored.clone();
            async move {
                let listed = pictures.list().await;
                let previews = previews_of(&picture_tiles(&listed, &stored));
                found.set(listed);
                make_smalls(pictures, previews, smalls).await;
            }
        }
    });
    let tiles = picture_tiles(&found.read(), &stored);
    let current = current_of(&tiles);
    let rows: Vec<Vec<Choice<String>>> = {
        let smalls = smalls.read();
        tiles
            .chunks(PER_ROW)
            .map(|row| row.iter().map(|tile| choice_of(tile, &smalls)).collect())
            .collect()
    };
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
                            make_smalls(pictures.clone(), vec![path], smalls).await;
                        }
                    });
                },
            }
        }
    }
}
