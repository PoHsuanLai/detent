//! The Displays page, as Settings meets it: each display with its resolution, refresh rate,
//! rotation and place beside the others. Pure; the compositor is the window's (`detent-ui`).

mod arrange;
mod looks;
mod model;
mod page;
mod step;
mod words;

pub use arrange::{ArrangeSide, place_beside, side_of};
pub use looks::{LooksLike, default_scale, looks_like};
pub use model::{
    DisplayMode, DisplayPosition, DisplayRotation, DisplaysStatus, Millimetres, ModePreference,
    Monitor, OutputId, PixelSize, RefreshMilliHz, ScaleMilli,
};
pub use page::{DISPLAYS_SLUG, displays_page, is_displays};
pub use step::{
    DisplayChange, DisplayEdit, DisplaysEffect, DisplaysInput, DisplaysReady, DisplaysState,
    displays_step,
};
pub use words::{hertz_words, missing_words};

#[cfg(test)]
mod tests;
