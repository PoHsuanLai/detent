//! The Bluetooth page: the switch, the devices this computer knows (My Devices) and the ones
//! nearby, with the buttons that connect, disconnect and forget them.

use super::drive::use_bluetooth;
use super::sheets::{ForgetSheet, PairingSheet};
use crate::system::Unavailable;
use detent_model::{
    BtDevice, BtInput, BtKind, BtLink, BtPower, BtReady, BtSheet, BtState, bluetooth_missing_words,
};
use dioxus::prelude::*;
use ds::components::content::label::LabelRole;
use ds::components::controls::button_model::Bezel;
use ds::components::fields::field_row::{FieldGroup, FieldRow};
use ds::prelude::*;
use ds::root::common::Common;

/// The page: the switch and the devices, or why there are none.
#[component]
pub(crate) fn BluetoothPane() -> Element {
    let driver = use_bluetooth();
    let state = driver.state.read().clone();
    match state {
        BtState::Looking => rsx! { div { class: "system-page", "data-system": "looking" } },
        BtState::Unavailable(missing) => rsx! {
            Unavailable { title: "Bluetooth", words: bluetooth_missing_words(missing) }
        },
        BtState::Ready(page) => {
            let press = EventHandler::new(move |input: BtInput| driver.feed(input));
            rsx! { Listed { page, press } }
        }
    }
}

fn power_of(check: Check) -> BtPower {
    match check {
        Check::On => BtPower::On,
        Check::Off | Check::Mixed => BtPower::Off,
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
    let on = page.status.power == BtPower::On;
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
    rsx! {
        div { class: "system-page", "data-system": "bluetooth",
            FieldGroup {
                FieldRow { label: TextLine::from("Bluetooth"),
                    Toggle {
                        label: "Bluetooth",
                        value: if on { Check::On } else { Check::Off },
                        onchange: move |next| press.call(BtInput::Switch(power_of(next))),
                    }
                }
            }
            if on {
                if !mine.is_empty() {
                    Devices { title: "My Devices", rows: mine, press }
                }
                Devices { title: "Nearby Devices", rows: nearby, press }
            }
            match page.sheet.clone() {
                BtSheet::Closed => rsx! {},
                BtSheet::Forget(address) => rsx! {
                    ForgetSheet {
                        name: page
                            .status
                            .device(&address)
                            .map_or_else(|| address.0.clone(), |device| device.name.clone()),
                        press,
                    }
                },
                BtSheet::Pairing(question) => rsx! { PairingSheet { question, press } },
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
                    label: TextLine::from("No devices found yet"),
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
        Group::Mine if work == RowWork::Working => BtLink::Connecting.words(),
        Group::Mine => device.link.words(),
        Group::Nearby => "Not Paired",
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
                    Label { text: level.words(), role: LabelRole::Secondary }
                }
            }
            match (group, device.link) {
                (Group::Mine, BtLink::Connected) => rsx! {
                    Button {
                        label: "Disconnect",
                        bezel: Bezel::Inline,
                        common: Common { aria_label: Some(format!("Disconnect {name}")), ..Common::default() },
                        onclick: {
                            let address = address.clone();
                            move |_| press.call(BtInput::Disconnect(address.clone()))
                        },
                    }
                },
                (Group::Mine, _) => rsx! {
                    Button {
                        label: "Connect",
                        bezel: Bezel::Inline,
                        availability,
                        common: Common { aria_label: Some(format!("Connect {name}")), ..Common::default() },
                        onclick: {
                            let address = address.clone();
                            move |_| press.call(BtInput::Connect(address.clone()))
                        },
                    }
                },
                (Group::Nearby, _) => rsx! {
                    Button {
                        label: "Connect",
                        bezel: Bezel::Inline,
                        availability,
                        common: Common { aria_label: Some(format!("Connect {name}")), ..Common::default() },
                        onclick: {
                            let address = address.clone();
                            move |_| press.call(BtInput::Pair(address.clone()))
                        },
                    }
                },
            }
            if group == Group::Mine {
                Button {
                    label: "Forget\u{2026}",
                    bezel: Bezel::Inline,
                    common: Common { aria_label: Some(format!("Forget {name}")), ..Common::default() },
                    onclick: move |_| press.call(BtInput::AskForget(address.clone())),
                }
            }
        }
    }
}
