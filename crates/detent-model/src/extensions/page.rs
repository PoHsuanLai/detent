//! The page's place among the pages: quire declares it by the id `extensions`.

use ds_settings::schema::Page;

/// The page's slug: what `--page` and `open-page` take.
pub const EXTENSIONS_SLUG: &str = "extensions";

/// The Extensions page as the catalogue holds it.
pub fn extensions_page() -> Page {
    Page::declared(EXTENSIONS_SLUG)
}
