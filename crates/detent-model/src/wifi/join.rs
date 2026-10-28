//! What a change of the wireless device's state means for a join, and how an access point's flags
//! say it is secured. Pure: the network service's numbers go in, facts come out.

use super::model::NetworkLink;

/// How a network is secured, from its access point's flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    /// Anyone may join.
    Open,
    /// A shared password, WPA-PSK.
    Psk,
    /// A shared password, WPA3.
    Sae,
    /// A sign-in with an account.
    Eap,
    /// An old scheme this page does not do.
    Unsupported,
}

/// The security `flags`, `wpa` and `rsn` of an access point describe.
pub fn wifi_security_of(flags: u32, wpa: u32, rsn: u32) -> WifiSecurity {
    const PRIVACY: u32 = 0x1;
    const PSK: u32 = 0x100;
    const ENTERPRISE: u32 = 0x200;
    const SAE: u32 = 0x400;
    let keys = wpa | rsn;
    if keys & ENTERPRISE != 0 && keys & (PSK | SAE) == 0 {
        WifiSecurity::Eap
    } else if keys & PSK != 0 {
        WifiSecurity::Psk
    } else if keys & SAE != 0 {
        WifiSecurity::Sae
    } else if flags & PRIVACY != 0 || keys != 0 {
        WifiSecurity::Unsupported
    } else {
        WifiSecurity::Open
    }
}

/// What the device's state says of the access point it is on: `NM_DEVICE_STATE_ACTIVATED` is
/// joined, the states from preparing to the last check are joining.
pub fn wifi_link_of(state: u32) -> NetworkLink {
    match state {
        ACTIVATED => NetworkLink::Joined,
        PREPARE..=SECONDARIES => NetworkLink::Joining,
        _ => NetworkLink::Idle,
    }
}

/// What a change of the device's state means for a join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinVerdict {
    /// The device is on the network.
    Done,
    /// The network did not accept the password.
    WrongPassword,
    /// The join failed for another reason.
    CouldNot,
    /// The device has started on the join.
    Going,
    /// Nothing to conclude yet.
    Waiting,
}

/// `NM_DEVICE_STATE_*` as a join passes through them.
const DISCONNECTED: u32 = 30;
const PREPARE: u32 = 40;
const NEED_AUTH: u32 = 60;
const SECONDARIES: u32 = 90;
const ACTIVATED: u32 = 100;
const FAILED: u32 = 120;

/// `NM_DEVICE_STATE_REASON_NO_SECRETS` to `_SUPPLICANT_TIMEOUT`: the network did not accept the
/// password.
const REFUSED_REASONS: std::ops::RangeInclusive<u32> = 7..=11;

/// The meaning of the device entering `state` for `reason`, given whether it had `started` on the
/// join.
pub fn join_verdict(state: u32, reason: u32, started: bool, security: WifiSecurity) -> JoinVerdict {
    let refused = security != WifiSecurity::Open && REFUSED_REASONS.contains(&reason);
    match state {
        ACTIVATED => JoinVerdict::Done,
        NEED_AUTH if security != WifiSecurity::Open => JoinVerdict::WrongPassword,
        FAILED if refused => JoinVerdict::WrongPassword,
        DISCONNECTED if refused && started => JoinVerdict::WrongPassword,
        FAILED => JoinVerdict::CouldNot,
        DISCONNECTED if started => JoinVerdict::CouldNot,
        PREPARE..=SECONDARIES => JoinVerdict::Going,
        _ => JoinVerdict::Waiting,
    }
}

#[cfg(test)]
mod tests {
    use super::{JoinVerdict, WifiSecurity, join_verdict, wifi_link_of, wifi_security_of};
    use crate::wifi::NetworkLink;

    #[test]
    fn flags_say_how_a_network_is_secured() {
        const CASES: &[(&str, u32, u32, u32, WifiSecurity)] = &[
            ("open", 0, 0, 0, WifiSecurity::Open),
            ("wpa2 personal", 1, 0, 0x108, WifiSecurity::Psk),
            ("wpa3", 1, 0, 0x400, WifiSecurity::Sae),
            ("wpa2 and wpa3", 1, 0, 0x500, WifiSecurity::Psk),
            ("enterprise", 1, 0, 0x200, WifiSecurity::Eap),
            ("wep", 1, 0, 0, WifiSecurity::Unsupported),
        ];
        for (case, flags, wpa, rsn, want) in CASES {
            assert_eq!(wifi_security_of(*flags, *wpa, *rsn), *want, "{case}");
        }
    }

    #[test]
    fn the_device_state_says_how_joined_the_active_network_is() {
        const CASES: &[(u32, NetworkLink)] = &[
            (30, NetworkLink::Idle),
            (40, NetworkLink::Joining),
            (60, NetworkLink::Joining),
            (90, NetworkLink::Joining),
            (100, NetworkLink::Joined),
            (120, NetworkLink::Idle),
        ];
        for (state, want) in CASES {
            assert_eq!(wifi_link_of(*state), *want, "{state}");
        }
    }

    #[test]
    fn the_device_state_ends_a_join_or_does_not() {
        use WifiSecurity::{Open, Psk, Sae};
        // (case, state, reason, started, security, verdict)
        const CASES: &[(&str, u32, u32, bool, WifiSecurity, JoinVerdict)] = &[
            ("on the network", 100, 0, false, Psk, JoinVerdict::Done),
            (
                "asked for a secret",
                60,
                0,
                true,
                Psk,
                JoinVerdict::WrongPassword,
            ),
            ("no secrets", 120, 7, true, Psk, JoinVerdict::WrongPassword),
            (
                "timed out in the handshake",
                120,
                11,
                true,
                Sae,
                JoinVerdict::WrongPassword,
            ),
            (
                "an open network cannot be wrong",
                120,
                7,
                true,
                Open,
                JoinVerdict::CouldNot,
            ),
            (
                "leaving the old network",
                30,
                0,
                false,
                Psk,
                JoinVerdict::Waiting,
            ),
            (
                "dropped after starting",
                30,
                0,
                true,
                Psk,
                JoinVerdict::CouldNot,
            ),
            ("preparing", 40, 0, false, Psk, JoinVerdict::Going),
        ];
        for (case, state, reason, started, security, want) in CASES {
            assert_eq!(
                join_verdict(*state, *reason, *started, *security),
                *want,
                "{case}"
            );
        }
    }
}
