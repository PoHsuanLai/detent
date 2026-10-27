//! The page's place among the pages: quire declares it by the id `displays`.

use ds_settings::schema::Page;

/// The page's slug: what `--page` and `open-page` take.
pub const DISPLAYS_SLUG: &str = "displays";

/// The Displays page as the catalogue holds it.
pub fn displays_page() -> Page {
    Page::declared(DISPLAYS_SLUG)
}
