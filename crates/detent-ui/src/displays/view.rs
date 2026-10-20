//! The Displays page: a card for each display with its resolution, the pixel size of its mode, its
//! refresh rate and rotation, and where it sits beside the others.

use super::drive::use_displays;
use crate::system::Unavailable;
use detent_model::{
    ArrangeSide, Brightness, BrightnessList, DisplayChange, DisplayEdit, DisplayRotation,
    DisplaysInput, DisplaysReady, DisplaysState, Monitor, OutputId, PixelSize, RefreshMilliHz,
    ScaleMilli, default_scale, display_size_words, displays_missing_words, hertz_words, looks_like,
    mode_sizes, native_size, side_of,
};
use dioxus::prelude::*;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The page: the displays, or why there are none.
#[component]
pub(crate) fn DisplaysPane() -> Element {
    let driver = use_displays();
    let state = driver.state.read().clone();
    match state {
        DisplaysState::Looking => rsx! { div { class: "system-page", "data-system": "looking" } },
        DisplaysState::Unavailable(missing) => rsx! {
            Unavailable { title: "Displays", words: displays_missing_words(missing) }
        },
        DisplaysState::Ready(page) => {
            let brightness = driver.brightness.read().clone();
            let dim = EventHandler::new({
                let driver = driver.clone();
                move |(output, level): (OutputId, Brightness)| driver.set_brightness(output, level)
            });
            let press = EventHandler::new(move |input: DisplaysInput| driver.feed(input));
            rsx! { Listed { page, brightness, press, dim } }
        }
    }
}

#[component]
fn Listed(
    page: DisplaysReady,
    brightness: BrightnessList,
    press: EventHandler<DisplaysInput>,
    dim: EventHandler<(OutputId, Brightness)>,
) -> Element {
    let availability = if page.is_applying() {
        Availability::Busy
    } else {
        Availability::Enabled
    };
    let displays = page.status.displays.clone();
    rsx! {
        div { class: "system-page", "data-system": "displays",
            for display in displays.clone() {
                DisplayCard {
                    key: "{display.id.0}",
                    others: displays.iter().filter(|other| other.id != display.id).cloned().collect::<Vec<_>>(),
                    level: brightness.level_of(&display.id),
                    display,
                    availability,
                    press,
                    dim,
                }
            }
        }
    }
}

/// One display's settings.
#[component]
fn DisplayCard(
    display: Monitor,
    others: Vec<Monitor>,
    level: Option<Brightness>,
    availability: Availability,
    press: EventHandler<DisplaysInput>,
    dim: EventHandler<(OutputId, Brightness)>,
) -> Element {
    let id = display.id.clone();
    let dimmed = display.id.clone();
    let change = EventHandler::new(move |edit: DisplayEdit| {
        press.call(DisplaysInput::Change(DisplayChange {
            output: id.clone(),
            edit,
        }));
    });
    rsx! {
        div { "data-display": "{display.id.0}",
            FieldGroup { title: Some(display.name.clone()),
                if let Some(level) = level {
                    BrightnessRow { level, dim: move |level| dim.call((dimmed.clone(), level)) }
                }
                ResolutionRow { display: display.clone(), availability, pick: move |scale| change.call(DisplayEdit::Scale(scale)) }
                RefreshRow { display: display.clone(), availability, pick: move |rate| change.call(DisplayEdit::Refresh(rate)) }
                RotationRow { display: display.clone(), availability, pick: move |turn| change.call(DisplayEdit::Rotate(turn)) }
                if !others.is_empty() {
                    PlaceRow { display: display.clone(), others, availability, press }
                }
                ModeRow { display: display.clone(), availability, pick: move |size| change.call(DisplayEdit::Size(size)) }
            }
        }
    }
}

/// The display's brightness, for a display the shell lists one for.
#[component]
fn BrightnessRow(level: Brightness, dim: EventHandler<Brightness>) -> Element {
    rsx! {
        FieldRow { label: TextLine::from("Brightness"), availability: Availability::Enabled,
            div { class: "key-slider",
                div { class: "key-slider-track",
                    Slider {
                        label: "Brightness".to_owned(),
                        value: Fraction(level.fraction()),
                        step: Fraction(10),
                        size: ControlSize::Small,
                        onchange: move |at: Fraction| {
                            let next = Brightness::of_fraction(at.0);
                            if next != level {
                                dim.call(next);
                            }
                        },
                    }
                }
            }
        }
    }
}

#[component]
fn ResolutionRow(
    display: Monitor,
    availability: Availability,
    pick: EventHandler<ScaleMilli>,
) -> Element {
    let default = default_scale(&display);
    let looks = looks_like(&display);
    let items: Vec<MenuItem<ScaleMilli>> = looks
        .iter()
        .map(|look| {
            let size = display_size_words(look.size);
            let label = if look.scale == default {
                format!("Default for display ({size})")
            } else {
                size
            };
            MenuItem::new(look.scale, label)
        })
        .collect();
    let now = looks
        .iter()
        .find(|look| look.scale == display.scale)
        .map(|look| format!("Looks like {}", display_size_words(look.size)));
    rsx! {
        FieldRow { label: TextLine::from("Resolution"), help: now.map(TextLine::from), availability,
            PopUpButton::<ScaleMilli> {
                items,
                value: Some(display.scale),
                size: ControlSize::Small,
                availability,
                onpick: move |scale| pick.call(scale),
            }
        }
    }
}

/// The pixel sizes the display can draw; a display with one size has nothing to choose.
#[component]
fn ModeRow(display: Monitor, availability: Availability, pick: EventHandler<PixelSize>) -> Element {
    let native = native_size(&display.modes);
    let sizes = mode_sizes(&display.modes);
    let alone = sizes.len() < 2;
    let items: Vec<MenuItem<PixelSize>> = sizes
        .iter()
        .map(|size| {
            let words = display_size_words(*size);
            let label = if native == Some(*size) {
                format!("{words} (Native)")
            } else {
                words
            };
            MenuItem::new(*size, label)
        })
        .collect();
    let availability = if alone {
        Availability::Disabled
    } else {
        availability
    };
    rsx! {
        FieldRow {
            label: TextLine::from("Display Mode"),
            help: Some(TextLine::from("The pixels the display draws")),
            availability,
            PopUpButton::<PixelSize> {
                items,
                value: display.current.map(|mode| mode.size),
                size: ControlSize::Small,
                availability,
                onpick: move |size| pick.call(size),
            }
        }
    }
}

/// The refresh rates the display's current size runs at, the fastest first.
fn rates_of(display: &Monitor) -> Vec<RefreshMilliHz> {
    let Some(current) = display.current else {
        return Vec::new();
    };
    let mut rates: Vec<RefreshMilliHz> = display
        .modes
        .iter()
        .filter(|mode| mode.size == current.size)
        .map(|mode| mode.refresh)
        .collect();
    rates.sort_by_key(|rate| std::cmp::Reverse(*rate));
    rates.dedup();
    rates
}

#[component]
fn RefreshRow(
    display: Monitor,
    availability: Availability,
    pick: EventHandler<RefreshMilliHz>,
) -> Element {
    let rates = rates_of(&display);
    let alone = rates.len() < 2;
    let items: Vec<MenuItem<RefreshMilliHz>> = rates
        .iter()
        .map(|rate| MenuItem::new(*rate, hertz_words(*rate)))
        .collect();
    rsx! {
        FieldRow {
            label: TextLine::from("Refresh Rate"),
            availability: if alone { Availability::Disabled } else { availability },
            PopUpButton::<RefreshMilliHz> {
                items,
                value: display.current.map(|mode| mode.refresh),
                size: ControlSize::Small,
                availability: if alone { Availability::Disabled } else { availability },
                onpick: move |rate| pick.call(rate),
            }
        }
    }
}

#[component]
fn RotationRow(
    display: Monitor,
    availability: Availability,
    pick: EventHandler<DisplayRotation>,
) -> Element {
    let items: Vec<MenuItem<DisplayRotation>> = DisplayRotation::ALL
        .iter()
        .map(|turn| MenuItem::new(*turn, turn.words()))
        .collect();
    rsx! {
        FieldRow { label: TextLine::from("Rotation"), availability,
            PopUpButton::<DisplayRotation> {
                items,
                value: Some(display.rotation),
                size: ControlSize::Small,
                availability,
                onpick: move |turn| pick.call(turn),
            }
        }
    }
}

/// Where this display sits: a side of one of the others.
#[component]
fn PlaceRow(
    display: Monitor,
    others: Vec<Monitor>,
    availability: Availability,
    press: EventHandler<DisplaysInput>,
) -> Element {
    let items: Vec<MenuItem<(OutputId, ArrangeSide)>> = others
        .iter()
        .flat_map(|other| {
            ArrangeSide::ALL.into_iter().map(move |side| {
                MenuItem::new(
                    (other.id.clone(), side),
                    format!("{} {}", side.words(), other.name),
                )
            })
        })
        .collect();
    let now = others
        .iter()
        .find_map(|other| side_of(&display, other).map(|side| (other.id.clone(), side)));
    let id = display.id.clone();
    rsx! {
        FieldRow { label: TextLine::from("Position"), availability,
            PopUpButton::<(OutputId, ArrangeSide)> {
                items,
                value: now,
                size: ControlSize::Small,
                availability,
                onpick: move |(beside, side): (OutputId, ArrangeSide)| {
                    press.call(DisplaysInput::Arrange(id.clone(), beside, side));
                },
            }
        }
    }
}
