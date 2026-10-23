//! Extensions as Settings meets them: the choices a `Provided` key offers (the Look menu), and
//! the Extensions page that lists what is installed with a switch each. Pure; the folders and
//! `quire/extensions.toml` are `detent-files`'.

mod list;
mod offers;
mod page;
mod words;

pub use list::{
    ExtensionGroup, ExtensionKeep, ExtensionList, ExtensionPower, ExtensionRow, ExtensionStanding,
    InstalledExtensions,
};
pub use offers::{Offer, Offers, stray_word};
pub use page::{EXTENSIONS_SLUG, extensions_page, is_extensions};
pub use words::{
    EXTENSIONS_EMPTY_HELP, EXTENSIONS_EMPTY_TITLE, NOT_AVAILABLE, NOT_INSTALLED,
    UNREADABLE_MANIFEST,
};
