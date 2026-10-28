//! The compositor's heads and modes as the page's displays, and back.

use desktop_client::output::{Enabled, Head, Mode, Outputs, Preferred, Transform};
use detent_model::{
    DisplayMode, DisplayPosition, DisplayRotation, Millimetres, Monitor, OutputId, PixelSize,
    RefreshMilliHz, ScaleMilli,
};

/// The displays that are switched on, in the order the compositor announced them.
pub fn monitors_of(outputs: &Outputs) -> Vec<Monitor> {
    outputs
        .heads()
        .filter(|head| head.enabled == Enabled::On)
        .map(monitor_of)
        .collect()
}

/// Connectors of a computer's own panel.
const INTERNAL: [&str; 3] = ["eDP", "LVDS", "DSI"];

/// What a person calls a head: a computer's own panel by that name, else its maker and model,
/// else what the compositor describes it as, else its connector.
fn name_of(head: &Head) -> String {
    let connector = head.name.as_str();
    if INTERNAL.iter().any(|prefix| connector.starts_with(prefix)) {
        return "Built-in Display".to_owned();
    }
    match (&head.make, &head.model) {
        (Some(make), Some(model)) if !make.is_empty() && !model.is_empty() => {
            format!("{make} {model}")
        }
        _ if !head.description.is_empty() => head.description.clone(),
        _ => connector.to_owned(),
    }
}

fn monitor_of(head: &Head) -> Monitor {
    let modes: Vec<DisplayMode> = head.modes.iter().map(mode_of).collect();
    Monitor {
        id: OutputId(head.name.as_str().to_owned()),
        name: name_of(head),
        current: head.current().map(mode_of),
        modes,
        scale: ScaleMilli(head.scale.get()),
        rotation: rotation_of(head.transform),
        position: DisplayPosition {
            x: head.position.x,
            y: head.position.y,
        },
        size_mm: head
            .physical_size_mm
            .filter(|size| size.w > 0 && size.h > 0)
            .map(|size| Millimetres {
                width: size.w,
                height: size.h,
            }),
    }
}

pub fn mode_of(mode: &Mode) -> DisplayMode {
    DisplayMode {
        size: PixelSize {
            width: mode.size.w,
            height: mode.size.h,
        },
        refresh: RefreshMilliHz(mode.refresh.get()),
        preferred: matches!(mode.preferred, Preferred::Yes),
    }
}

/// A flipped head reads as its rotation: the page turns displays and does not mirror them.
fn rotation_of(transform: Transform) -> DisplayRotation {
    match transform {
        Transform::Normal | Transform::Flipped => DisplayRotation::Standard,
        Transform::Rotate90 | Transform::Flipped90 => DisplayRotation::Quarter,
        Transform::Rotate180 | Transform::Flipped180 => DisplayRotation::Half,
        Transform::Rotate270 | Transform::Flipped270 => DisplayRotation::ThreeQuarters,
        _ => DisplayRotation::Standard,
    }
}

/// The transform that turns a head to `rotation`.
pub fn transform_of(rotation: DisplayRotation) -> Transform {
    match rotation {
        DisplayRotation::Standard => Transform::Normal,
        DisplayRotation::Quarter => Transform::Rotate90,
        DisplayRotation::Half => Transform::Rotate180,
        DisplayRotation::ThreeQuarters => Transform::Rotate270,
    }
}

#[cfg(test)]
mod tests {
    use super::{rotation_of, transform_of};
    use desktop_client::output::Transform;
    use detent_model::DisplayRotation;

    #[test]
    fn a_rotation_goes_to_the_transform_and_back() {
        for rotation in DisplayRotation::ALL {
            assert_eq!(
                rotation_of(transform_of(rotation)),
                rotation,
                "{rotation:?}"
            );
        }
        assert_eq!(rotation_of(Transform::Flipped90), DisplayRotation::Quarter);
    }
}
