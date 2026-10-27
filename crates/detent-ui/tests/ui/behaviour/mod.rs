//! The window driven through the harness: pages, toggles, choices and search, on scratch
//! directories. Every write is read back from the scratch config home.

use crate::support::Sandbox;
use crate::support::drive::{click, page_of, search_for, sidebar_page, title_of};
use detent_files::load;
use detent_model::PageSlug;
use detent_ui::Opening;
use ds::prelude::Word;
use ds::prelude::{Point, Px};
use ds_harness::Input;
use std::time::Duration;

mod controls;
mod pages;
mod search;

fn opening(page: &str) -> Opening {
    Opening {
        page: Some(PageSlug(page.to_owned())),
        query: String::new(),
    }
}

fn harness_spec(sandbox: &Sandbox) -> detent_model::FileSpec {
    sandbox
        .env(Opening::default(), ds::prelude::Scheme::Light)
        .catalogue
        .files
        .iter()
        .find(|f| {
            f.file.relative().ends_with("settings.toml") && f.file.relative().starts_with("sill")
        })
        .cloned()
        .expect("sill's file")
}

fn detent_model_key(path: &str) -> ds_settings::schema::KeyPath {
    ds_settings::schema::KeyPath(path.to_owned())
}
