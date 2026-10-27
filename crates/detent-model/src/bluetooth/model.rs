//! What the Bluetooth page knows: the switch and the devices BlueZ reports, and a question a
//! device asks while pairing. Parsed once where it enters.

/// A device's address, as BlueZ writes it (`AA:BB:CC:DD:EE:FF`): what names a device.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BtAddress(pub String);

/// The Bluetooth switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtPower {
    /// Off.
    Off,
    /// On.
    On,
}

/// A device's relation to this computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtLink {
    /// Not connected.
    Disconnected,
    /// Connecting now.
    Connecting,
    /// Connected.
    Connected,
}

/// Whether this computer has paired with a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtPairing {
    /// It has.
    Paired,
    /// It has not: the device was only seen.
    Unpaired,
}

/// What a device is, for its glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtKind {
    /// Headphones, a headset, a speaker.
    Audio,
    /// A mouse or a trackpad.
    Mouse,
    /// A keyboard.
    Keyboard,
    /// A game controller.
    Gamepad,
    /// A phone.
    Phone,
    /// Anything else.
    Other,
}

impl BtKind {
    /// The kind BlueZ's icon name describes.
    pub fn of_icon(icon: &str) -> BtKind {
        match icon {
            "audio-headset" | "audio-headphones" | "audio-card" | "audio-speakers" => BtKind::Audio,
            "input-mouse" | "input-tablet" => BtKind::Mouse,
            "input-keyboard" => BtKind::Keyboard,
            "input-gaming" => BtKind::Gamepad,
            "phone" => BtKind::Phone,
            _ => BtKind::Other,
        }
    }
}

/// A device's battery, in percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryLevel(pub u8);

/// One device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtDevice {
    /// Its address.
    pub address: BtAddress,
    /// Its name.
    pub name: String,
    /// What it is.
    pub kind: BtKind,
    /// Whether it is connected.
    pub link: BtLink,
    /// Whether this computer is paired with it.
    pub pairing: BtPairing,
    /// Its battery, when it tells.
    pub battery: Option<BatteryLevel>,
}

/// The switch and the devices, the connected ones first, then by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BtStatus {
    /// The switch.
    pub power: BtPower,
    /// The devices, in the order the page lists them.
    pub devices: Vec<BtDevice>,
}

impl BtStatus {
    /// The status of `devices` under `power`, sorted.
    pub fn new(power: BtPower, mut devices: Vec<BtDevice>) -> BtStatus {
        devices.sort_by(|a, b| {
            let rank = |d: &BtDevice| d.link == BtLink::Disconnected;
            rank(a)
                .cmp(&rank(b))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.address.cmp(&b.address))
        });
        BtStatus { power, devices }
    }

    /// The devices this computer is paired with: "My Devices".
    pub fn mine(&self) -> impl Iterator<Item = &BtDevice> {
        self.devices
            .iter()
            .filter(|d| d.pairing == BtPairing::Paired)
    }

    /// The devices it only sees: "Nearby Devices".
    pub fn nearby(&self) -> impl Iterator<Item = &BtDevice> {
        self.devices
            .iter()
            .filter(|d| d.pairing == BtPairing::Unpaired)
    }

    /// The device at `address`.
    pub fn device(&self, address: &BtAddress) -> Option<&BtDevice> {
        self.devices.iter().find(|d| &d.address == address)
    }
}

/// The six-digit number two devices compare while pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairingCode(pub u32);

/// What a device asks of the person while pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingAsk {
    /// Is this number the one on the device?
    Compare(PairingCode),
    /// May it pair?
    Allow,
    /// Type this number on the device. Nothing is answered here.
    Show(PairingCode),
    /// Type the PIN the device is set to (a keyboard shows one, an older device has a fixed one).
    PinCode,
    /// Type the six-digit number the device shows.
    Passkey,
}

/// The most characters a PIN has: BlueZ takes sixteen.
const PIN_LONGEST: usize = 16;
/// The most digits a passkey has.
const PASSKEY_LONGEST: usize = 6;

impl PairingAsk {
    /// Whether the person types the answer: a PIN or a passkey.
    pub fn takes_text(self) -> bool {
        matches!(self, PairingAsk::PinCode | PairingAsk::Passkey)
    }

    /// Whether `text` is a PIN or a passkey this question takes; a question that wants no typing
    /// takes nothing.
    pub fn accepts(self, text: &str) -> bool {
        let length = text.chars().count();
        match self {
            PairingAsk::PinCode => (1..=PIN_LONGEST).contains(&length),
            PairingAsk::Passkey => {
                (1..=PASSKEY_LONGEST).contains(&length) && text.chars().all(|c| c.is_ascii_digit())
            }
            PairingAsk::Compare(_) | PairingAsk::Allow | PairingAsk::Show(_) => false,
        }
    }
}

/// A pairing question, for the device that asks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairingQuestion {
    /// The device.
    pub address: BtAddress,
    /// Its name.
    pub name: String,
    /// What it asks.
    pub ask: PairingAsk,
}

/// The person's answer to a pairing question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PairingAnswer {
    /// Yes: the number matches, or pairing is allowed.
    Yes,
    /// No.
    No,
    /// The PIN or the passkey the person typed.
    Typed(String),
}
