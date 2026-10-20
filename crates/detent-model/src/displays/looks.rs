//! The resolutions a display offers: sizes it can look like at the scales the page allows. The
//! compositor lists modes and takes a scale; "looks like 1440 × 900" is the mode over the scale.

use super::model::{Millimetres, Monitor, PixelSize, ScaleMilli};

/// The scales offered, in thousandths.
const SCALES: [u32; 7] = [1000, 1250, 1500, 1750, 2000, 2500, 3000];

/// The narrowest a display may look: below this nothing fits.
const NARROWEST: u32 = 640;

/// A density outside this range is a size the display got wrong, not a display.
const PLAUSIBLE_PPI: std::ops::RangeInclusive<u32> = 40..=600;

/// The density a default looks right at, in points per inch.
const COMFORTABLE_PPI: u32 = 115;

/// One resolution on offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LooksLike {
    /// The scale that makes it.
    pub scale: ScaleMilli,
    /// What the display looks like at that scale, in logical pixels.
    pub size: PixelSize,
}

fn looks_at(pixels: PixelSize, scale: u32) -> PixelSize {
    let over = |pixels: u32| {
        u32::try_from(u64::from(pixels) * 1000 / u64::from(scale)).unwrap_or(u32::MAX)
    };
    PixelSize {
        width: over(pixels.width),
        height: over(pixels.height),
    }
}

/// The density of `pixels` spread over `size`, along the diagonal; `None` when the size is
/// missing or implausible.
fn density(pixels: PixelSize, size: Option<Millimetres>) -> Option<u32> {
    let size = size?;
    let diagonal_px = f64::from(pixels.width).hypot(f64::from(pixels.height));
    let diagonal_in = f64::from(size.width).hypot(f64::from(size.height)) / 25.4;
    if diagonal_in <= 0.0 {
        return None;
    }
    let ppi = (diagonal_px / diagonal_in).round();
    let ppi = u32::try_from(ppi as i64).ok()?;
    PLAUSIBLE_PPI.contains(&ppi).then_some(ppi)
}

/// The scale the display looks right at: the one that brings its density nearest
/// [`COMFORTABLE_PPI`], and 1 for a display that does not say how big it is.
pub fn default_scale(display: &Monitor) -> ScaleMilli {
    let Some(mode) = display.current else {
        return ScaleMilli(SCALES[0]);
    };
    let Some(ppi) = density(mode.size, display.size_mm) else {
        return ScaleMilli(SCALES[0]);
    };
    let wanted = ppi * 1000 / COMFORTABLE_PPI;
    let nearest = SCALES
        .iter()
        .copied()
        .min_by_key(|scale| scale.abs_diff(wanted))
        .unwrap_or(SCALES[0]);
    ScaleMilli(nearest)
}

/// The resolutions the display can look like, the widest first. The default is among them.
pub fn looks_like(display: &Monitor) -> Vec<LooksLike> {
    let Some(mode) = display.current else {
        return Vec::new();
    };
    let default = default_scale(display);
    let mut looks: Vec<LooksLike> = SCALES
        .iter()
        .copied()
        .map(|scale| LooksLike {
            scale: ScaleMilli(scale),
            size: looks_at(mode.size, scale),
        })
        .filter(|look| look.size.width >= NARROWEST || look.scale == default)
        .collect();
    looks.sort_by_key(|look| std::cmp::Reverse(look.size));
    looks
}
