//! The small drawn window each theme choice shows: a window on the desktop in the light palette,
//! in the dark one, or half of each for the choice that follows the desktop. Every colour is a
//! quire token read in the palette the picture stands for, so the picture keeps its own palette
//! whichever one the Settings window is in.

use ds::prelude::Scheme;
use ds::style::icon::url::IconUrl;
use ds::style::tokens::colour::ColourToken;

/// The side of the square picture, in the SVG's own units and in pixels on screen.
pub(super) const SIDE: u8 = 64;

/// The palettes of a theme word's picture, left half then right half: the same one twice for
/// light and dark, one of each for the word that follows the desktop. None for any other word.
pub(super) fn palettes(word: &str) -> Option<(Scheme, Scheme)> {
    match word {
        "light" => Some((Scheme::Light, Scheme::Light)),
        "dark" => Some((Scheme::Dark, Scheme::Dark)),
        "system" => Some((Scheme::Light, Scheme::Dark)),
        _ => None,
    }
}

/// `token` in `scheme`, as the colour an SVG paints with.
fn paint(token: ColourToken, scheme: Scheme) -> String {
    token.value(scheme).css()
}

/// The desktop and the window on it in `scheme`, clipped to the strip `x..x + width`.
fn scene(scheme: Scheme, x: u8, width: u8) -> String {
    let desktop = paint(ColourToken::Accent, scheme);
    let window = paint(ColourToken::Surface, scheme);
    let side = paint(ColourToken::Surface2, scheme);
    let line = paint(ColourToken::Line, scheme);
    let faint = paint(ColourToken::InkFaint, scheme);
    let clip = format!("clip{x}");
    format!(
        concat!(
            r#"<clipPath id="{clip}"><rect x="{x}" y="0" width="{width}" height="64"/></clipPath>"#,
            r#"<g clip-path="url(#{clip})">"#,
            r#"<linearGradient id="sky{x}" x1="0" y1="0" x2="0" y2="1">"#,
            r#"<stop offset="0" stop-color="{desktop}" stop-opacity="0.55"/>"#,
            r#"<stop offset="1" stop-color="{desktop}"/></linearGradient>"#,
            r#"<rect width="64" height="64" fill="url(#sky{x})"/>"#,
            r#"<rect x="8" y="14" width="48" height="36" rx="4" fill="{window}" stroke="{line}"/>"#,
            r#"<rect x="9" y="24" width="14" height="22" fill="{side}"/>"#,
            r#"<circle cx="13" cy="19" r="1.5" fill="{faint}"/>"#,
            r#"<circle cx="18" cy="19" r="1.5" fill="{faint}"/>"#,
            r#"<circle cx="23" cy="19" r="1.5" fill="{faint}"/>"#,
            r#"<rect x="27" y="27" width="24" height="3" rx="1.5" fill="{faint}"/>"#,
            r#"<rect x="27" y="34" width="18" height="3" rx="1.5" fill="{faint}"/>"#,
            r#"<rect x="27" y="41" width="21" height="3" rx="1.5" fill="{faint}"/>"#,
            r#"</g>"#,
        ),
        clip = clip,
        x = x,
        width = width,
        desktop = desktop,
        window = window,
        side = side,
        line = line,
        faint = faint,
    )
}

/// The picture of a theme word as an SVG `data:` URL; none for a word that is not a theme.
pub(super) fn picture_of(word: &str) -> Option<IconUrl> {
    let (left, right) = palettes(word)?;
    let half = SIDE / 2;
    // The whole picture is a rounded card, as a thumbnail in System Settings is.
    let document = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{SIDE}" height="{SIDE}" viewBox="0 0 64 64">"#,
            r#"<clipPath id="card"><rect width="64" height="64" rx="10"/></clipPath>"#,
            r#"<g clip-path="url(#card)">{}{}</g></svg>"#,
        ),
        scene(left, 0, half),
        scene(right, half, half),
        SIDE = SIDE,
    );
    Some(IconUrl::svg(&document))
}

#[cfg(test)]
mod tests {
    use super::{palettes, picture_of};
    use ds::prelude::Scheme;

    #[test]
    fn each_theme_word_has_a_picture_and_any_other_has_none() {
        for word in ["light", "dark", "system"] {
            assert!(picture_of(word).is_some(), "{word}");
        }
        assert!(picture_of("blue").is_none());
        assert_eq!(palettes("system"), Some((Scheme::Light, Scheme::Dark)));
    }

    #[test]
    fn the_pictures_differ() {
        let urls: Vec<_> = ["light", "dark", "system"]
            .iter()
            .filter_map(|word| picture_of(word))
            .collect();
        assert!(urls[0] != urls[1] && urls[1] != urls[2] && urls[0] != urls[2]);
    }
}
