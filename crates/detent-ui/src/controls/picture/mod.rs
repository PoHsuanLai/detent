//! A picture: the Wallpaper page as the Mac draws it. A row at the top shows the picture on the
//! desktop, larger, with its name and the "Add Picture..." button; below it the person's own
//! pictures, when there are any, and the system's, each a grid of tiles with the current one
//! selected.

mod grid;
mod header;

use crate::copy::pictures::add_failure;
use crate::live::use_client;
use detent_client::Wallpaper;
use detent_model::{AddOutcome, Picture, PictureSections, picture_sections};
use dioxus::prelude::*;
use grid::pictures_grid;
use header::picture_header;
use std::collections::HashMap;
use std::path::PathBuf;

/// The small pictures made so far, by the file each is made from.
pub(super) type Smalls = HashMap<PathBuf, PathBuf>;

/// What the person calls the pictures they added.
const YOURS: &str = "Your Pictures";

/// What the person calls the pictures the system ships.
const SYSTEMS: &str = "Desktop Pictures";

/// Makes the small picture of each of `previews` not in `smalls` yet, one after the other so the
/// first tiles fill in first, and puts each in `smalls` as it is made.
async fn make_smalls(pictures: Wallpaper, previews: Vec<PathBuf>, mut smalls: Signal<Smalls>) {
    for preview in previews {
        if smalls.peek().contains_key(&preview) {
            continue;
        }
        if let Some(small) = pictures.thumbnail(preview.clone()).await {
            smalls.write().insert(preview, small);
        }
    }
}

/// The files the small pictures are made from, in the order they are seen: the current picture,
/// the added ones, the system's.
fn previews_of(sections: &PictureSections) -> Vec<PathBuf> {
    sections
        .current
        .iter()
        .chain(&sections.added)
        .chain(&sections.system)
        .map(|tile| tile.preview.clone())
        .collect()
}

/// The pictures on offer and the button that adds one; a pick or an added picture is sent to
/// `onchange` as the path to store.
#[component]
pub fn PictureControl(
    label: String,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let pictures = use_client().wallpaper();
    let mut found = use_signal(Vec::<Picture>::new);
    let smalls = use_signal(Smalls::new);
    let mut told = use_signal(|| None::<String>);
    let stored = value.as_str().unwrap_or_default().to_owned();
    use_future({
        let pictures = pictures.clone();
        let stored = stored.clone();
        move || {
            let pictures = pictures.clone();
            let stored = stored.clone();
            async move {
                let listed = pictures.list().await;
                let previews = previews_of(&picture_sections(&listed, &stored));
                found.set(listed);
                make_smalls(pictures, previews, smalls).await;
            }
        }
    });
    let sections = picture_sections(&found.read(), &stored);
    let pick = move |path: String| onchange.call(toml::Value::String(path));
    let onadd = move |_: ()| {
        let pictures = pictures.clone();
        spawn(async move {
            let outcome = pictures.add().await;
            told.set(add_failure(&outcome));
            if let AddOutcome::Added(path) = outcome {
                onchange.call(toml::Value::String(path.to_string_lossy().into_owned()));
                found.set(pictures.list().await);
                make_smalls(pictures.clone(), vec![path], smalls).await;
            }
        });
    };
    let smalls = smalls.read();
    rsx! {
        div { class: "key-pictures", "aria-label": label,
            {picture_header(sections.current.as_ref(), &smalls, told(), EventHandler::new(onadd))}
            if !sections.added.is_empty() {
                {pictures_grid(YOURS, &sections.added, &smalls, EventHandler::new(pick))}
            }
            {pictures_grid(SYSTEMS, &sections.system, &smalls, EventHandler::new(pick))}
        }
    }
}
