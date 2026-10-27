//! The Displays page, as Settings meets it: each display with its resolution, refresh rate,
//! rotation and place beside the others. Pure; the compositor is the window's (`detent-ui`).

mod arrange;
mod brightness;
mod looks;
mod model;
mod modes;
mod page;
mod step;

pub use arrange::{ArrangeSide, place_beside, side_of};
pub use brightness::{Brightness, BrightnessList};
pub use looks::{LooksLike, default_scale, looks_like};
pub use model::{
    DisplayMode, DisplayPosition, DisplayRotation, DisplaysStatus, Millimetres, ModePreference,
    Monitor, OutputId, PixelSize, RefreshMilliHz, ScaleMilli,
};
pub use modes::{mode_at, mode_sizes, native_size};
pub use page::{DISPLAYS_SLUG, displays_page};
pub use step::{
    DisplayChange, DisplayEdit, DisplaysEffect, DisplaysInput, DisplaysReady, DisplaysState,
    displays_step,
};

#[cfg(test)]
mod tests;
