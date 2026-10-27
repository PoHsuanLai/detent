//! A bounded number: a slider with its value beside it.

use detent_model::Range;
use detent_model::external::toml;
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The value's share of the slider's travel that one arrow key moves: one unit of a whole range,
/// a hundredth of a fractional one.
fn key_step(range: &Range) -> Fraction {
    let span = range.max.saturating_sub(range.min).max(1);
    Fraction((1000 / span.min(1000)).max(1) as u16)
}

/// A slider over `range`. Every move is sent, but only a move to another value: a whole range
/// far narrower than the slider's thousand steps writes once per unit, not once per pixel.
#[component]
pub fn SliderControl(
    label: String,
    range: Range,
    value: toml::Value,
    onchange: EventHandler<toml::Value>,
) -> Element {
    let position = Fraction(range.position(&value));
    let step = key_step(&range);
    let shown = range.display(&value);
    let current = value.clone();
    let edit = range.clone();
    rsx! {
        div { class: "key-slider",
            div { class: "key-slider-track",
                Slider {
                    label,
                    value: position,
                    step,
                    size: ControlSize::Small,
                    onchange: move |at: Fraction| {
                        let next = edit.value_at(at.0);
                        if next != current {
                            onchange.call(next);
                        }
                    },
                }
            }
            Label { text: shown, role: LabelRole::Secondary }
        }
    }
}
