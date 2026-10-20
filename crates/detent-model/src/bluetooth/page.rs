//! The page's place among the pages: quire declares it by the id `bluetooth`.

use ds_settings::schema::Page;

/// The page's slug: what `--page` and `open-page` take.
pub const BLUETOOTH_SLUG: &str = "bluetooth";

/// The Bluetooth page as the catalogue holds it.
pub fn bluetooth_page() -> Page {
    Page::declared(BLUETOOTH_SLUG)
}

/// Whether `page` is the Bluetooth page.
pub fn is_bluetooth(page: &Page) -> bool {
    *page == bluetooth_page()
}
