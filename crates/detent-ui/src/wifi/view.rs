//! The Wi-Fi page: the switch, the network this computer is on with its Details, and the networks
//! in range, each with its lock and its signal and a Join button.

use super::drive::use_wifi;
use super::sheets::{DetailsSheet, PasswordSheet};
use crate::system::Unavailable;
use detent_model::{
    NetworkLock, Remembered, SignalBars, Ssid, WIFI_BLOCKED, WifiInput, WifiNetwork, WifiRadio,
    WifiReady, WifiSheet, WifiState, wifi_missing_words,
};
use dioxus::prelude::*;
use ds::components::controls::button_model::Bezel;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;

/// The page: the switch and the networks, or why there are none.
#[component]
pub(crate) fn WifiPane() -> Element {
    let driver = use_wifi();
    let state = driver.state.read().clone();
    match state {
        WifiState::Looking => rsx! { div { class: "system-page", "data-system": "looking" } },
        WifiState::Unavailable(missing) => rsx! {
            Unavailable { title: "Wi-Fi", words: wifi_missing_words(missing) }
        },
        WifiState::Ready(page) => {
            let press = EventHandler::new(move |input: WifiInput| driver.feed(input));
            rsx! { Listed { page, press } }
        }
    }
}

fn radio_of(check: Check) -> WifiRadio {
    match check {
        Check::On => WifiRadio::On,
        Check::Off | Check::Mixed => WifiRadio::Off,
    }
}

#[component]
fn Listed(page: WifiReady, press: EventHandler<WifiInput>) -> Element {
    let radio = page.status.radio;
    let blocked = radio == WifiRadio::Blocked;
    let availability = if blocked {
        Availability::Disabled
    } else {
        Availability::Enabled
    };
    let help = blocked.then(|| TextLine::from(WIFI_BLOCKED));
    let others = |wanted: Remembered| -> Vec<WifiNetwork> {
        page.status
            .others()
            .filter(|network| network.remembered == wanted)
            .cloned()
            .collect()
    };
    let (known, new) = (others(Remembered::Yes), others(Remembered::No));
    let busy = page.joining.clone();
    rsx! {
        div { class: "system-page", "data-system": "wifi",
            FieldGroup {
                FieldRow { label: TextLine::from("Wi-Fi"), help, availability,
                    Toggle {
                        label: "Wi-Fi",
                        value: if radio == WifiRadio::On { Check::On } else { Check::Off },
                        availability,
                        onchange: move |next| press.call(WifiInput::Switch(radio_of(next))),
                    }
                }
            }
            if radio == WifiRadio::On {
                if let Some(joined) = page.status.joined().cloned() {
                    JoinedNetwork { network: joined, press }
                }
                if !known.is_empty() {
                    Networks { title: "Known Networks", networks: known, busy: busy.clone(), press }
                }
                if !new.is_empty() {
                    Networks { title: "Other Networks", networks: new, busy: busy.clone(), press }
                }
            }
            match page.sheet.clone() {
                WifiSheet::Closed => rsx! {},
                WifiSheet::Password(ask) => rsx! { PasswordSheet { ask, press } },
                WifiSheet::Details(ssid) => rsx! {
                    if let Some(network) = page.status.network(&ssid).cloned() {
                        DetailsSheet { network, press }
                    }
                },
            }
        }
    }
}

/// A network's lock, when it has one, and its signal.
#[component]
fn Glyphs(network: WifiNetwork) -> Element {
    let bars = match network.strength.signal() {
        SignalBars::Weak => Icon::WifiLow,
        SignalBars::Fair => Icon::WifiHigh,
        SignalBars::Good => Icon::Wifi,
    };
    let locked = network.lock == NetworkLock::Secured;
    rsx! {
        div {
            class: "system-glyphs",
            "data-lock": if locked { "secured" } else { "open" },
            "data-signal": network.strength.signal().words().to_lowercase(),
            if locked {
                IconView { source: IconSource::from(Icon::Lock) }
            }
            IconView { source: IconSource::from(bars) }
        }
    }
}

/// The network this computer is on, with its Details.
#[component]
fn JoinedNetwork(network: WifiNetwork, press: EventHandler<WifiInput>) -> Element {
    rsx! {
        FieldGroup {
            FieldRow {
                label: TextLine::from(network.ssid.0.clone()),
                help: Some(TextLine::from("Connected")),
                common: Common { id: Some("wifi-joined".to_owned()), ..Common::default() },
                Glyphs { network: network.clone() }
                Button {
                    label: "Details\u{2026}",
                    bezel: Bezel::Push,
                    common: Common { aria_label: Some("Details\u{2026}".to_owned()), ..Common::default() },
                    onclick: move |_| press.call(WifiInput::OpenDetails),
                }
            }
        }
    }
}

/// Whether the page is busy with this row, another row, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowWork {
    /// A press joins it.
    Free,
    /// It is being joined.
    Joining,
    /// Another network is being joined, so a press waits.
    Held,
}

/// A card of networks, each with a Join button.
#[component]
fn Networks(
    title: String,
    networks: Vec<WifiNetwork>,
    busy: Option<Ssid>,
    press: EventHandler<WifiInput>,
) -> Element {
    rsx! {
        FieldGroup { title: Some(title),
            for network in networks {
                NetworkRow {
                    key: "{network.ssid.0}",
                    work: match &busy {
                        Some(ssid) if *ssid == network.ssid => RowWork::Joining,
                        Some(_) => RowWork::Held,
                        None => RowWork::Free,
                    },
                    network,
                    press,
                }
            }
        }
    }
}

#[component]
fn NetworkRow(network: WifiNetwork, work: RowWork, press: EventHandler<WifiInput>) -> Element {
    let ssid = network.ssid.clone();
    let (availability, help) = match work {
        RowWork::Free => (Availability::Enabled, None),
        RowWork::Joining => (
            Availability::Busy,
            Some(TextLine::from("Connecting\u{2026}")),
        ),
        RowWork::Held => (Availability::Disabled, None),
    };
    rsx! {
        FieldRow { label: TextLine::from(network.ssid.0.clone()), help,
            Glyphs { network: network.clone() }
            Button {
                label: "Join",
                bezel: Bezel::Inline,
                availability,
                common: Common { aria_label: Some(format!("Join {}", ssid.0)), ..Common::default() },
                onclick: move |_| press.call(WifiInput::Pick(ssid.clone())),
            }
        }
    }
}
