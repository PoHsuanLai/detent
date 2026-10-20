//! The Wi-Fi page as a machine: what the service said, what the person pressed, and what each
//! press asks of the service.

use super::account::Account;
use super::hidden::HiddenNetwork;
use super::model::{NetworkLink, NetworkLock, Remembered, Ssid, WifiRadio, WifiStatus};
use super::passphrase::Passphrase;
use super::words::OTHER_FAILED;
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
    /// The password of a network, or the username and password of one that signs in.
    Password(PasswordAsk),
    /// The details of the network this computer is on.
    Details(Ssid),
    /// Whether to forget a network.
    Forget(Ssid),
    /// The name, security and password of a network that does not announce itself.
    Other,
}

/// The question for a network's password.
#[derive(Debug, Clone, PartialEq)]
pub struct PasswordAsk {
    /// The network.
    pub ssid: Ssid,
    /// Whether this is the first try.
    pub standing: AskStanding,
    /// What is asked for.
    pub kind: AskKind,
}

/// What a network wants from the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AskKind {
    /// A password.
    Password,
    /// A username and a password.
    Account,
}

impl AskKind {
    /// What a network with this `lock` wants when it has to be asked.
    fn of(lock: NetworkLock) -> AskKind {
        match lock {
            NetworkLock::Enterprise => AskKind::Account,
            NetworkLock::Open | NetworkLock::Secured => AskKind::Password,
        }
    }
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
    /// The person gave the username and password of a network that signs in and pressed Join.
    SignIn(Ssid, Account),
    /// The person opened the sheet for a network that does not announce itself.
    OpenOther,
    /// The person filled that sheet and pressed Join.
    JoinOther(HiddenNetwork),
    /// The person pressed Forget on a network they have joined before.
    AskForget(Ssid),
    /// The person confirmed forgetting.
    ConfirmForget,
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
    /// Join `ssid` with the account the person gave.
    SignIn(Ssid, Account),
    /// Join a network that does not announce its name.
    JoinHidden(HiddenNetwork),
    /// Remove the kept password of `ssid`.
    Forget(Ssid),
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
        WifiInput::SignIn(ssid, account) => (
            WifiReady {
                sheet: WifiSheet::Closed,
                joining: Some(ssid.clone()),
                ..page
            },
            vec![WifiEffect::SignIn(ssid, account)],
        ),
        WifiInput::OpenOther => (with_sheet(page, WifiSheet::Other), Vec::new()),
        WifiInput::JoinOther(network) => (
            WifiReady {
                sheet: WifiSheet::Closed,
                joining: Some(network.ssid.clone()),
                ..page
            },
            vec![WifiEffect::JoinHidden(network)],
        ),
        WifiInput::AskForget(ssid) => (with_sheet(page, WifiSheet::Forget(ssid)), Vec::new()),
        WifiInput::ConfirmForget => confirm_forget(page),
        WifiInput::Joined(ssid, Ok(())) => (without_joining(page, &ssid), Vec::new()),
        WifiInput::Joined(ssid, Err(JoinFailure::WrongPassword)) => ask_again(page, ssid),
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

fn with_sheet(page: WifiReady, sheet: WifiSheet) -> WifiReady {
    WifiReady { sheet, ..page }
}

fn confirm_forget(page: WifiReady) -> (WifiReady, Vec<WifiEffect>) {
    match page.sheet.clone() {
        WifiSheet::Forget(ssid) => (
            with_sheet(page, WifiSheet::Closed),
            vec![WifiEffect::Forget(ssid)],
        ),
        WifiSheet::Closed | WifiSheet::Password(_) | WifiSheet::Details(_) | WifiSheet::Other => {
            (page, Vec::new())
        }
    }
}

/// A network refused what it was given: a network in the list is asked again, one that was typed
/// in is not (the name, the security or the password may be what is wrong).
fn ask_again(page: WifiReady, ssid: Ssid) -> (WifiReady, Vec<WifiEffect>) {
    let page = without_joining(page, &ssid);
    match page.status.network(&ssid).map(|network| network.lock) {
        Some(lock) => {
            let ask = PasswordAsk {
                ssid,
                standing: AskStanding::Wrong,
                kind: AskKind::of(lock),
            };
            (with_sheet(page, WifiSheet::Password(ask)), Vec::new())
        }
        None => (page, vec![WifiEffect::Tell(OTHER_FAILED.to_owned())]),
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
    let ask = PasswordAsk {
        ssid,
        standing: AskStanding::First,
        kind: AskKind::of(network.lock),
    };
    (with_sheet(page, WifiSheet::Password(ask)), Vec::new())
}

fn open_details(page: WifiReady) -> (WifiReady, Vec<WifiEffect>) {
    let sheet = page.status.joined().map_or(WifiSheet::Closed, |network| {
        WifiSheet::Details(network.ssid.clone())
    });
    (WifiReady { sheet, ..page }, Vec::new())
}
