//! The Wi-Fi page: the switch, the network this computer is on with its Details, and the networks
//! in range, each with its lock and its signal and a Join button; known ones can be forgotten, and
//! a network that hides its name is joined from Other.

use super::network_sheets::{ForgetSheet, OtherSheet};
use super::sheets::{DetailsSheet, PasswordSheet};
use crate::copy;
use crate::copy::absence::Subject;
use crate::icons::tile_of;
use crate::live::use_live;
use crate::system::{RowWork, Unavailable};
use detent_client::Client;
use detent_model::{
    PageKind, SignalBars, Ssid, WifiAsking, WifiInput, WifiNetwork, WifiRadio, WifiReady,
    WifiSecurity, WifiState,
};
use dioxus::prelude::*;
use ds::components::controls::button_model::Bezel;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;

/// The page: the switch and the networks, or why there are none.
#[component]
pub(crate) fn WifiPane() -> Element {
    let wifi = use_live(Client::wifi);
    let state = wifi.state.read().clone();
    match state {
        WifiState::Looking => rsx! { div { class: "system-page", "data-system": "looking" } },
        WifiState::Unavailable(absence) => rsx! {
            Unavailable { face: tile_of(PageKind::Wifi), title: crate::copy::wifi::WI_FI, subject: Subject::Wifi, absence }
        },
        WifiState::Ready(page) => {
            let press = EventHandler::new(move |input: WifiInput| {
                spawn(wifi.pane.send(input));
            });
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
    let joined = page.status.joined().map(|network| network.ssid.0.clone());
    let help = Some(TextLine::from(copy::wifi::status_line(
        radio,
        joined.as_deref(),
    )));
    let others = |wanted: bool| -> Vec<WifiNetwork> {
        page.status
            .others()
            .filter(|network| network.remembered == wanted)
            .cloned()
            .collect()
    };
    let (known, new) = (others(true), others(false));
    let busy = page.joining.clone();
    rsx! {
        div { class: "system-page", "data-system": "wifi",
            FieldGroup {
                FieldRow {
                    label: TextLine::from(crate::copy::wifi::WI_FI),
                    help,
                    leading: Some(tile_of(PageKind::Wifi)),
                    availability,
                    Toggle {
                        label: crate::copy::wifi::WI_FI,
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
                    Networks { title: crate::copy::wifi::KNOWN_NETWORKS, networks: known, busy: busy.clone(), other_row: false, press }
                }
                Networks { title: crate::copy::wifi::OTHER_NETWORKS, networks: new, busy: busy.clone(), other_row: true, press }
            }
            match page.asking.clone() {
                WifiAsking::Nothing => rsx! {},
                WifiAsking::Password(ask) => rsx! { PasswordSheet { ask, press } },
                WifiAsking::Details(ssid) => rsx! {
                    if let Some(network) = page.status.network(&ssid).cloned() {
                        DetailsSheet { network, press }
                    }
                },
                WifiAsking::Forget(ssid) => rsx! { ForgetSheet { name: ssid.0, press } },
                WifiAsking::Other => rsx! { OtherSheet { press } },
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
    let locked = network.security != WifiSecurity::Open;
    rsx! {
        div {
            class: "system-glyphs",
            "data-lock": if locked { "secured" } else { "open" },
            "data-signal": copy::wifi::signal(network.strength.signal()).to_lowercase(),
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
                help: Some(TextLine::from(crate::copy::wifi::CONNECTED)),
                common: Common { id: Some("wifi-joined".to_owned()), ..Common::default() },
                Glyphs { network: network.clone() }
                Button {
                    label: crate::copy::wifi::DETAILS_ELLIPSIS,
                    bezel: Bezel::Push,
                    common: Common { aria_label: Some(crate::copy::wifi::DETAILS_ELLIPSIS.to_owned()), ..Common::default() },
                    onclick: move |_| press.call(WifiInput::OpenDetails),
                }
            }
        }
    }
}

/// A card of networks, each with a Join button.
#[component]
fn Networks(
    title: String,
    networks: Vec<WifiNetwork>,
    busy: Option<Ssid>,
    other_row: bool,
    press: EventHandler<WifiInput>,
) -> Element {
    rsx! {
        FieldGroup { title: Some(title),
            for network in networks {
                NetworkRow {
                    key: "{network.ssid.0}",
                    work: match &busy {
                        Some(ssid) if *ssid == network.ssid => RowWork::Working,
                        Some(_) => RowWork::Held,
                        None => RowWork::Free,
                    },
                    network,
                    press,
                }
            }
            if other_row {
                OtherRow { press }
            }
        }
    }
}

/// The row that opens the sheet for a network that does not announce its name.
#[component]
fn OtherRow(press: EventHandler<WifiInput>) -> Element {
    rsx! {
        FieldRow {
            label: TextLine::from(crate::copy::wifi::OTHER_ELLIPSIS),
            help: Some(TextLine::from(crate::copy::wifi::JOIN_A_NETWORK_BY_ITS_NAME)),
            Button {
                label: crate::copy::wifi::CONNECT_ELLIPSIS,
                bezel: Bezel::Inline,
                common: Common { aria_label: Some(crate::copy::wifi::JOIN_ANOTHER_NETWORK.to_owned()), ..Common::default() },
                onclick: move |_| press.call(WifiInput::OpenOther),
            }
        }
    }
}

#[component]
fn NetworkRow(network: WifiNetwork, work: RowWork, press: EventHandler<WifiInput>) -> Element {
    let ssid = network.ssid.clone();
    let availability = work.availability();
    let help =
        (work == RowWork::Working).then(|| TextLine::from(crate::copy::wifi::CONNECTING_ELLIPSIS));
    rsx! {
        FieldRow { label: TextLine::from(network.ssid.0.clone()), help,
            Glyphs { network: network.clone() }
            Button {
                label: crate::copy::wifi::JOIN,
                bezel: Bezel::Inline,
                availability,
                common: Common { aria_label: Some(crate::copy::wifi::join(&ssid.0)), ..Common::default() },
                onclick: {
                    let ssid = ssid.clone();
                    move |_| press.call(WifiInput::Pick(ssid.clone()))
                },
            }
            if network.remembered {
                Button {
                    label: crate::copy::common::FORGET_ELLIPSIS,
                    bezel: Bezel::Inline,
                    common: Common { aria_label: Some(crate::copy::wifi::forget(&ssid.0)), ..Common::default() },
                    onclick: move |_| press.call(WifiInput::AskForget(ssid.clone())),
                }
            }
        }
    }
}
