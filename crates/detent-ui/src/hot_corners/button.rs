//! The button at the foot of the page.

use super::sheet::HotCornersSheet;
use crate::files::{Edit, Files};
use detent_model::KeyEntry;
use dioxus::prelude::*;
use ds::prelude::*;
use ds::root::common::Common;

/// "Hot Corners…", and the sheet while it is open. `entries` are the four corner keys.
#[component]
pub(crate) fn HotCorners(
    entries: Vec<KeyEntry>,
    files: Signal<Files>,
    onedit: EventHandler<Edit>,
) -> Element {
    let mut opened = use_signal(|| Shown::Hidden);
    let close = EventHandler::new(move |()| opened.set(Shown::Hidden));
    rsx! {
        div { class: "hot-corners-button",
            Button {
                label: crate::copy::hot_corners::HOT_CORNERS_ELLIPSIS,
                common: Common { id: Some("hot-corners-open".to_owned()), ..Common::default() },
                onclick: move |_| opened.set(Shown::Visible),
            }
        }
        if opened() == Shown::Visible {
            HotCornersSheet { entries, files, onedit, onclose: close }
        }
    }
}
