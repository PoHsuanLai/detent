//! The page's place among the pages, and the slug of its first-run step. Quire's fixed page list
//! has no Assistant Model, so the catalogue holds it as the page of a program of that name, as it
//! does Software Update. The first-run step is the same page drawn as a short dialogue; it is
//! reached by its slug and never listed.

use ds_settings::schema::Page;

/// The name a person reads for the page.
pub(crate) const ASSISTANT_MODEL_NAME: &str = "Assistant Model";

/// The page's slug: what `--page` and `open-page` take.
pub const ASSISTANT_MODEL_SLUG: &str = "assistant-model";

/// The slug of the first-run step: the same picker with a short explanation, Continue and Skip.
pub const ASSISTANT_SETUP_SLUG: &str = "assistant-model-setup";

/// The Assistant Model page as the catalogue holds it.
pub fn assistant_model_page() -> Page {
    Page::App(ASSISTANT_MODEL_NAME.to_owned())
}
