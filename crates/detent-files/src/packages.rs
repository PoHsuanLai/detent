//! A wallpaper package, the folder layout KDE's wallpapers use: `metadata.json` naming the
//! wallpaper, `contents/images/<width>x<height>.<ending>` holding one file per screen size,
//! `contents/images_dark/` the same for a dark desktop, and `contents/screenshot.<ending>` a
//! small picture of it. The package is one picture to the person, not one per size.

use detent_model::{Picture, is_picture};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The file that marks a folder as a package.
const METADATA: &str = "metadata.json";

/// The folder of the sizes, under `contents`.
const IMAGES: &str = "images";

/// The endings a package's own screenshot may have, in the order they are tried.
const SCREENSHOT_ENDINGS: &[&str] = &["png", "jpg", "jpeg"];

/// The package in `folder`, or `None` when the folder is not one (no `metadata.json`, or no
/// picture under `contents/images`).
pub(crate) fn package_in(folder: &Path) -> Option<Picture> {
    let metadata = folder.join(METADATA);
    if !metadata.is_file() {
        return None;
    }
    let image = largest_image(&folder.join("contents").join(IMAGES))?;
    let title = title_of(&metadata).or_else(|| folder_name(folder))?;
    Some(Picture::packaged(image, title, screenshot_in(folder)))
}

/// The name the package gives itself (`KPlugin.Name`), when the file is JSON and has one.
fn title_of(metadata: &Path) -> Option<String> {
    let text = std::fs::read_to_string(metadata).ok()?;
    let json: Value = serde_json::from_str(&text).ok()?;
    let name = json.get("KPlugin")?.get("Name")?.as_str()?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// The folder's own name, for a package that does not name itself.
fn folder_name(folder: &Path) -> Option<String> {
    Some(folder.file_name()?.to_str()?.to_owned())
}

/// The picture of the most pixels in `images`, by the size its name gives (`5120x2880`); among
/// names that give none, the last by name.
fn largest_image(images: &Path) -> Option<PathBuf> {
    let read = std::fs::read_dir(images).ok()?;
    read.filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| path.is_file() && is_picture(path))
        .max_by_key(|path| (pixels_of(path), path.clone()))
}

/// The pixels a file's name promises: `<width>x<height>` before the ending, else none.
fn pixels_of(path: &Path) -> u64 {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return 0;
    };
    let Some((width, height)) = stem.split_once('x') else {
        return 0;
    };
    match (width.parse::<u64>(), height.parse::<u64>()) {
        (Ok(width), Ok(height)) => width.saturating_mul(height),
        (Err(_), _) | (_, Err(_)) => 0,
    }
}

/// The package's own small picture, when it ships one.
fn screenshot_in(folder: &Path) -> Option<PathBuf> {
    SCREENSHOT_ENDINGS
        .iter()
        .map(|ending| folder.join("contents").join(format!("screenshot.{ending}")))
        .find(|path| path.is_file())
}
