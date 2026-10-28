//! The Bluetooth page: the switch, the devices this computer knows (My Devices) and the ones
//! nearby, with the buttons that connect, disconnect and forget them.

use super::sheets::{ForgetSheet, PairingSheet};
use crate::copy;
use crate::copy::absence::Subject;
use crate::icons::tile_of;
use crate::live::use_live;
use crate::system::Unavailable;
use detent_client::Client;
use detent_model::{BtAsking, BtDevice, BtInput, BtKind, BtLink, BtReady, BtState, PageKind};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::controls::button_model::Bezel;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;

/// The page: the switch and the devices, or why there are none.
#[component]
pub(crate) fn BluetoothPane() -> Element {
    let bluetooth = use_live(Client::bluetooth);
    let state = bluetooth.state.read().clone();
    match state {
        BtState::Looking => rsx! { div { class: "system-page", "data-system": "looking" } },
        BtState::Unavailable(absence) => rsx! {
            Unavailable { face: tile_of(PageKind::Bluetooth), title: crate::copy::bluetooth::BLUETOOTH, subject: Subject::Bluetooth, absence }
        },
        BtState::Ready(page) => {
            let press = EventHandler::new(move |input: BtInput| {
                spawn(bluetooth.pane.send(input));
            });
            rsx! { Listed { page, press } }
        }
    }
}

fn glyph(kind: BtKind) -> Icon {
    match kind {
        BtKind::Audio => Icon::Headphones,
        BtKind::Mouse => Icon::Mouse,
        BtKind::Keyboard => Icon::Keyboard,
        BtKind::Gamepad => Icon::Gamepad,
        BtKind::Phone => Icon::Phone,
        BtKind::Other => Icon::Bluetooth,
    }
}

/// Whether the page is busy with a device, another device, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowWork {
    /// A press starts something.
    Free,
    /// This device is being connected or paired.
    Working,
    /// Another device is, so a press waits.
    Held,
}

/// What the buttons of a row do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Group {
    /// A device this computer is paired with.
    Mine,
    /// A device it only sees.
    Nearby,
}

#[component]
fn Listed(page: BtReady, press: EventHandler<BtInput>) -> Element {
    let on = page.status.powered;
    let work_of = |device: &BtDevice| {
        if page.is_working(&device.address) {
            RowWork::Working
        } else if page.busy.is_some() {
            RowWork::Held
        } else {
            RowWork::Free
        }
    };
    let rows = |devices: Vec<BtDevice>, group: Group| -> Vec<(BtDevice, RowWork, Group)> {
        devices
            .into_iter()
            .map(|device| {
                let work = work_of(&device);
                (device, work, group)
            })
            .collect()
    };
    let mine = rows(page.status.mine().cloned().collect(), Group::Mine);
    let nearby = rows(page.status.nearby().cloned().collect(), Group::Nearby);
    let connected = page
        .status
        .mine()
        .filter(|device| device.link == BtLink::Connected)
        .count();
    let status = crate::copy::bluetooth::status_line(page.status.powered, connected);
    rsx! {
        div { class: "system-page", "data-system": "bluetooth",
            FieldGroup {
                FieldRow {
                    label: TextLine::from(crate::copy::bluetooth::BLUETOOTH),
                    help: Some(TextLine::from(status)),
                    leading: Some(tile_of(PageKind::Bluetooth)),
                    Toggle {
                        label: crate::copy::bluetooth::BLUETOOTH,
                        value: if on { Check::On } else { Check::Off },
                        onchange: move |next| press.call(BtInput::Switch(matches!(next, Check::On))),
                    }
                }
            }
            if on {
                if !mine.is_empty() {
                    Devices { title: crate::copy::bluetooth::MY_DEVICES, rows: mine, press }
                }
                Devices { title: crate::copy::bluetooth::NEARBY_DEVICES, rows: nearby, press }
            }
            match page.asking.clone() {
                BtAsking::Nothing => rsx! {},
                BtAsking::Forget(address) => rsx! {
                    ForgetSheet {
                        name: page
                            .status
                            .device(&address)
                            .map_or_else(|| address.0.clone(), |device| device.name.clone()),
                        press,
                    }
                },
                BtAsking::Pairing(question) => rsx! { PairingSheet { question, press } },
            }
        }
    }
}

#[component]
fn Devices(
    title: String,
    rows: Vec<(BtDevice, RowWork, Group)>,
    press: EventHandler<BtInput>,
) -> Element {
    rsx! {
        FieldGroup { title: Some(title),
            if rows.is_empty() {
                FieldRow {
                    label: TextLine::from(crate::copy::bluetooth::NO_DEVICES_FOUND_YET),
                    availability: Availability::Disabled,
                    Label { text: "Looking\u{2026}", role: LabelRole::Secondary }
                }
            }
            for (device, work, group) in rows {
                DeviceRow { key: "{device.address.0}", device, work, group, press }
            }
        }
    }
}

#[component]
fn DeviceRow(
    device: BtDevice,
    work: RowWork,
    group: Group,
    press: EventHandler<BtInput>,
) -> Element {
    let help = match group {
        Group::Mine if work == RowWork::Working => copy::bluetooth::link(BtLink::Connecting),
        Group::Mine => copy::bluetooth::link(device.link),
        Group::Nearby => crate::copy::bluetooth::NOT_PAIRED,
    };
    let availability = match work {
        RowWork::Free => Availability::Enabled,
        RowWork::Working => Availability::Busy,
        RowWork::Held => Availability::Disabled,
    };
    let (address, name) = (device.address.clone(), device.name.clone());
    rsx! {
        FieldRow {
            label: TextLine::from(device.name.clone()),
            help: Some(TextLine::from(help)),
            div { class: "system-glyphs", "data-kind": "{device.kind:?}",
                IconView { source: IconSource::from(glyph(device.kind)) }
            }
            if let Some(level) = device.battery {
                div { class: "system-battery", "data-system": "battery",
                    Label { text: copy::bluetooth::battery(level), role: LabelRole::Secondary }
                }
            }
            match (group, device.link) {
                (Group::Mine, BtLink::Connected) => rsx! {
                    Button {
                        label: crate::copy::bluetooth::DISCONNECT,
                        bezel: Bezel::Inline,
                        common: Common { aria_label: Some(crate::copy::bluetooth::disconnect(&name)), ..Common::default() },
                        onclick: {
                            let address = address.clone();
                            move |_| press.call(BtInput::Disconnect(address.clone()))
                        },
                    }
                },
                (Group::Mine, _) => rsx! {
                    Button {
                        label: crate::copy::bluetooth::CONNECT,
                        bezel: Bezel::Inline,
                        availability,
                        common: Common { aria_label: Some(crate::copy::bluetooth::connect(&name)), ..Common::default() },
                        onclick: {
                            let address = address.clone();
                            move |_| press.call(BtInput::Connect(address.clone()))
                        },
                    }
                },
                (Group::Nearby, _) => rsx! {
                    Button {
                        label: crate::copy::bluetooth::CONNECT,
                        bezel: Bezel::Inline,
                        availability,
                        common: Common { aria_label: Some(crate::copy::bluetooth::connect(&name)), ..Common::default() },
                        onclick: {
                            let address = address.clone();
                            move |_| press.call(BtInput::Pair(address.clone()))
                        },
                    }
                },
            }
            if group == Group::Mine {
                Button {
                    label: crate::copy::common::FORGET_ELLIPSIS,
                    bezel: Bezel::Inline,
                    common: Common { aria_label: Some(crate::copy::bluetooth::forget(&name)), ..Common::default() },
                    onclick: move |_| press.call(BtInput::AskForget(address.clone())),
                }
            }
        }
    }
}
