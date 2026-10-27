//! The page's place among the pages. Quire's fixed page list has no Software Update, so the
//! catalogue holds it as the page of a program of that name: its label, its slug and its place in
//! the sidebar are all decided here.

use ds_settings::schema::Page;

/// The name a person reads for the page.
pub(crate) const SOFTWARE_UPDATE_NAME: &str = "Software Update";

/// The page's slug: what `--page` takes.
pub(crate) const SOFTWARE_UPDATE_SLUG: &str = "software-update";

/// The Software Update page as the catalogue holds it.
pub(crate) fn software_update_page() -> Page {
    Page::App(SOFTWARE_UPDATE_NAME.to_owned())
}
