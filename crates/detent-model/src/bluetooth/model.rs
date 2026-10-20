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

impl std::fmt::Display for PairingCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:06}", self.0 % 1_000_000)
    }
}

/// What a device asks of the person while pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingAsk {
    /// Is this number the one on the device?
    Compare(PairingCode),
    /// May it pair?
    Allow,
    /// Type this number on the device. Nothing is answered here.
    Show(PairingCode),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingAnswer {
    /// Yes: the number matches, or pairing is allowed.
    Yes,
    /// No.
    No,
}
