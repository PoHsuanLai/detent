//! What the Wi-Fi page knows: the radio and the access points the network service reports,
//! folded into one row per network. Parsed once where it enters.

use super::join::WifiSecurity;
use std::collections::BTreeMap;

/// A network's name. SSIDs are bytes on the air; a name that is not text is shown lossily.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ssid(pub String);

/// How strong a signal is, in percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignalStrength(pub u8);

/// How a signal reads to a person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalBars {
    /// One arc.
    Weak,
    /// Two arcs.
    Fair,
    /// Three arcs.
    Good,
}

impl SignalStrength {
    /// The signal this strength is drawn as.
    pub fn signal(self) -> SignalBars {
        match self.0 {
            0..=33 => SignalBars::Weak,
            34..=66 => SignalBars::Fair,
            _ => SignalBars::Good,
        }
    }
}

/// The Wi-Fi switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiRadio {
    /// Switched off in software.
    Off,
    /// Switched on.
    On,
    /// Off in hardware (a key or a switch on the computer); the page cannot undo it.
    Blocked,
}

/// A network's relation to this computer. The order is the order of how joined it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NetworkLink {
    /// Not on it.
    Idle,
    /// Joining it now.
    Joining,
    /// On it.
    Joined,
}

/// One access point as the service reports it; a network may have several.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessPoint {
    /// The network it belongs to; empty for a hidden one.
    pub ssid: Ssid,
    /// Its signal.
    pub strength: SignalStrength,
    /// How it is secured.
    pub security: WifiSecurity,
    /// Whether this computer is on it.
    pub link: NetworkLink,
}

/// What a read of the network service gives, before it is folded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiWire {
    /// The switch.
    pub radio: WifiRadio,
    /// Every access point in range.
    pub access_points: Vec<AccessPoint>,
    /// The networks with a kept password.
    pub saved: Vec<Ssid>,
}

/// One network in the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiNetwork {
    /// Its name.
    pub ssid: Ssid,
    /// The signal of its strongest access point.
    pub strength: SignalStrength,
    /// How it is secured.
    pub security: WifiSecurity,
    /// Whether this computer is on it.
    pub link: NetworkLink,
    /// Whether its password is kept.
    pub remembered: bool,
}

/// The switch and the networks in range, the one this computer is on first, then the ones it
/// remembers, then by signal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WifiStatus {
    /// The switch.
    pub radio: WifiRadio,
    /// The networks, in the order the page lists them.
    pub networks: Vec<WifiNetwork>,
}

impl WifiStatus {
    /// The status `wire` describes: one network per name (hidden ones left out), sorted.
    pub fn of_wire(wire: WifiWire) -> WifiStatus {
        let mut folded: BTreeMap<Ssid, WifiNetwork> = BTreeMap::new();
        for point in wire
            .access_points
            .into_iter()
            .filter(|p| !p.ssid.0.is_empty())
        {
            let remembered = wire.saved.contains(&point.ssid);
            folded
                .entry(point.ssid.clone())
                .and_modify(|seen| {
                    seen.link = seen.link.max(point.link);
                    seen.strength = seen.strength.max(point.strength);
                })
                .or_insert(WifiNetwork {
                    ssid: point.ssid,
                    strength: point.strength,
                    security: point.security,
                    link: point.link,
                    remembered,
                });
        }
        let mut networks: Vec<WifiNetwork> = folded.into_values().collect();
        networks.sort_by(|a, b| {
            let rank = |n: &WifiNetwork| (n.link != NetworkLink::Joined, !n.remembered);
            rank(a)
                .cmp(&rank(b))
                .then(b.strength.cmp(&a.strength))
                .then(a.ssid.cmp(&b.ssid))
        });
        WifiStatus {
            radio: wire.radio,
            networks,
        }
    }

    /// The network this computer is on.
    pub fn joined(&self) -> Option<&WifiNetwork> {
        self.networks.iter().find(|n| n.link == NetworkLink::Joined)
    }

    /// The networks it is not on, in list order.
    pub fn others(&self) -> impl Iterator<Item = &WifiNetwork> {
        self.networks
            .iter()
            .filter(|n| n.link != NetworkLink::Joined)
    }

    /// The network called `ssid`.
    pub fn network(&self, ssid: &Ssid) -> Option<&WifiNetwork> {
        self.networks.iter().find(|n| &n.ssid == ssid)
    }
}
