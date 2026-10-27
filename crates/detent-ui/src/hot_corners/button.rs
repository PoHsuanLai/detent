//! The row at the foot of the page that opens the sheet: one drill-in row in a card of its own.

use super::sheet::HotCornersSheet;
use crate::files::{Edit, Files};
use detent_model::KeyEntry;
use dioxus::prelude::*;
use ds::components::fields::field_row::FieldGroup;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds::root::common::Common;

/// "Hot Corners…" as a row with a chevron, and the sheet while it is open. `entries` are the
/// four corner keys.
#[component]
pub(crate) fn HotCorners(
    entries: Vec<KeyEntry>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let mut opened = use_signal(|| Shown::Hidden);
    let close = EventHandler::new(move |()| opened.set(Shown::Hidden));
    rsx! {
        FieldGroup {
            Row {
                title: TextLine::from(crate::copy::hot_corners::HOT_CORNERS_ELLIPSIS),
                accessory: Accessory::Chevron,
                size: RowSize::Settings,
                common: Common { id: Some("hot-corners-open".to_owned()), ..Common::default() },
                onclick: move |_| opened.set(Shown::Visible),
            }
        }
        if opened() == Shown::Visible {
            HotCornersSheet { entries, files, onedit, onclose: close }
        }
    }
}
