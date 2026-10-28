//! Extensions as Settings meets them: the choices a `Provided` key offers (the Look menu), and
//! the Extensions page that lists what is installed with a switch each. Pure; the folders and
//! `quire/extensions.toml` are `detent-files`'.

mod list;
mod offers;
mod page;

pub use list::{
    ExtensionGroup, ExtensionKind, ExtensionList, ExtensionRow, ExtensionStanding,
    ExtensionTrouble, InstalledExtensions,
};
pub use offers::{Offer, Offers, stray_word};
pub use page::{EXTENSIONS_SLUG, extensions_page};
