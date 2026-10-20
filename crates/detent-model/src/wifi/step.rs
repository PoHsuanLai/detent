//! The Wi-Fi page as a machine: what the service said, what the person pressed, and what each
//! press asks of the service.

use super::model::{NetworkLink, NetworkLock, Remembered, Ssid, WifiRadio, WifiStatus};
use super::passphrase::Passphrase;
use crate::missing::Missing;

/// Where the page stands.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum WifiState {
    /// Nothing heard yet.
    #[default]
    Looking,
    /// There is nothing to show, for this reason.
    Unavailable(Missing),
    /// The switch and the networks.
    Ready(WifiReady),
}

/// The page with something to show.
#[derive(Debug, Clone, PartialEq)]
pub struct WifiReady {
    /// What the service last said.
    pub status: WifiStatus,
    /// The sheet over the page, if any.
    pub sheet: WifiSheet,
    /// The network a join was started for and has not ended.
    pub joining: Option<Ssid>,
}

/// A sheet over the page.
#[derive(Debug, Clone, PartialEq)]
pub enum WifiSheet {
    /// None.
    Closed,
    /// The password of a network.
    Password(PasswordAsk),
    /// The details of the network this computer is on.
    Details(Ssid),
}

/// The question for a network's password.
#[derive(Debug, Clone, PartialEq)]
pub struct PasswordAsk {
    /// The network.
    pub ssid: Ssid,
    /// Whether this is the first try.
    pub standing: AskStanding,
}

/// Whether the password was already tried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskStanding {
    /// The first time it is asked.
    First,
    /// The last one was refused.
    Wrong,
}

/// Why a join did not end on the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinFailure {
    /// The network refused the password.
    WrongPassword,
    /// Anything else, in a sentence for the person.
    Told(String),
}

/// What the page hears.
#[derive(Debug, Clone)]
pub enum WifiInput {
    /// A read of the service, or why there is none.
    Heard(Result<WifiStatus, Missing>),
    /// The person moved the switch.
    Switch(WifiRadio),
    /// The person chose a network to join.
    Pick(Ssid),
    /// The person gave the password and pressed Join.
    Password(Ssid, Passphrase),
    /// A join ended.
    Joined(Ssid, Result<(), JoinFailure>),
    /// The person opened the details of the network they are on.
    OpenDetails,
    /// The person closed the sheet.
    CloseSheet,
    /// The service refused a request, in these words.
    Refused(String),
}

/// What the page asks of the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WifiEffect {
    /// Move the switch.
    SetRadio(WifiRadio),
    /// Join `ssid`, with the password when one was given.
    Join(Ssid, Option<Passphrase>),
    /// Say this to the person.
    Tell(String),
}

fn ready(state: WifiState) -> Option<WifiReady> {
    match state {
        WifiState::Ready(ready) => Some(ready),
        WifiState::Looking | WifiState::Unavailable(_) => None,
    }
}

/// The next state, and what to ask of the service.
pub fn wifi_step(state: WifiState, input: WifiInput) -> (WifiState, Vec<WifiEffect>) {
    if let WifiInput::Heard(heard) = input {
        return (heard_step(state, heard), Vec::new());
    }
    match state {
        WifiState::Ready(page) => {
            let (page, effects) = press_step(page, input);
            (WifiState::Ready(page), effects)
        }
        other => (other, Vec::new()),
    }
}

fn heard_step(state: WifiState, heard: Result<WifiStatus, Missing>) -> WifiState {
    let status = match heard {
        Ok(status) => status,
        Err(missing) => return WifiState::Unavailable(missing),
    };
    let before = ready(state);
    let sheet = before
        .as_ref()
        .map_or(WifiSheet::Closed, |page| page.sheet.clone());
    let joining = before.and_then(|page| page.joining).filter(|ssid| {
        status
            .network(ssid)
            .is_some_and(|n| n.link != NetworkLink::Joined)
    });
    WifiState::Ready(WifiReady {
        status,
        sheet,
        joining,
    })
}

fn press_step(page: WifiReady, input: WifiInput) -> (WifiReady, Vec<WifiEffect>) {
    match input {
        WifiInput::Heard(_) => (page, Vec::new()),
        WifiInput::Switch(radio) => (page, vec![WifiEffect::SetRadio(radio)]),
        WifiInput::Pick(ssid) => pick(page, ssid),
        WifiInput::Password(ssid, passphrase) => (
            WifiReady {
                sheet: WifiSheet::Closed,
                joining: Some(ssid.clone()),
                ..page
            },
            vec![WifiEffect::Join(ssid, Some(passphrase))],
        ),
        WifiInput::Joined(ssid, Ok(())) => (without_joining(page, &ssid), Vec::new()),
        WifiInput::Joined(ssid, Err(JoinFailure::WrongPassword)) => (
            WifiReady {
                sheet: WifiSheet::Password(PasswordAsk {
                    ssid: ssid.clone(),
                    standing: AskStanding::Wrong,
                }),
                ..without_joining(page, &ssid)
            },
            Vec::new(),
        ),
        WifiInput::Joined(ssid, Err(JoinFailure::Told(words))) => {
            (without_joining(page, &ssid), vec![WifiEffect::Tell(words)])
        }
        WifiInput::OpenDetails => open_details(page),
        WifiInput::CloseSheet => (
            WifiReady {
                sheet: WifiSheet::Closed,
                ..page
            },
            Vec::new(),
        ),
        WifiInput::Refused(words) => (page, vec![WifiEffect::Tell(words)]),
    }
}

fn without_joining(page: WifiReady, ssid: &Ssid) -> WifiReady {
    let joining = page.joining.filter(|joining| joining != ssid);
    WifiReady { joining, ..page }
}

/// A network that needs no password, or whose password is kept, is joined at once; any other is
/// asked for its password first.
fn pick(page: WifiReady, ssid: Ssid) -> (WifiReady, Vec<WifiEffect>) {
    let Some(network) = page.status.network(&ssid).cloned() else {
        return (page, Vec::new());
    };
    if network.link != NetworkLink::Idle || page.joining.is_some() {
        return (page, Vec::new());
    }
    if network.lock == NetworkLock::Open || network.remembered == Remembered::Yes {
        let effects = vec![WifiEffect::Join(ssid.clone(), None)];
        return (
            WifiReady {
                joining: Some(ssid),
                ..page
            },
            effects,
        );
    }
    (
        WifiReady {
            sheet: WifiSheet::Password(PasswordAsk {
                ssid,
                standing: AskStanding::First,
            }),
            ..page
        },
        Vec::new(),
    )
}

fn open_details(page: WifiReady) -> (WifiReady, Vec<WifiEffect>) {
    let sheet = page.status.joined().map_or(WifiSheet::Closed, |network| {
        WifiSheet::Details(network.ssid.clone())
    });
    (WifiReady { sheet, ..page }, Vec::new())
}
