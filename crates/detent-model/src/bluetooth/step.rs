//! The Bluetooth page as a machine.

use super::model::{
    BtAddress, BtLink, BtPairing, BtPower, BtStatus, PairingAnswer, PairingAsk, PairingQuestion,
};
use crate::Absence;
use crate::Live;

/// Where the page stands: the switch and the devices once read.
pub type BtState = Live<BtReady>;

/// The page with something to show.
#[derive(Debug, Clone, PartialEq)]
pub struct BtReady {
    /// What BlueZ last said.
    pub status: BtStatus,
    /// What the person is asked over the page, if anything.
    pub asking: BtAsking,
    /// The device a connect or a pairing was started for and has not ended.
    pub busy: Option<BtAddress>,
}

/// What the person is asked over the page.
#[derive(Debug, Clone, PartialEq)]
pub enum BtAsking {
    /// Nothing.
    Nothing,
    /// Are you sure you want to forget this device?
    Forget(BtAddress),
    /// A device asks something while pairing.
    Pairing(PairingQuestion),
}

/// What the page hears.
#[derive(Debug, Clone)]
pub enum BtInput {
    /// A read of BlueZ, or why there is none.
    Heard(Result<BtStatus, Absence>),
    /// The person moved the switch.
    Switch(BtPower),
    /// The person pressed Connect on a device they have paired with.
    Connect(BtAddress),
    /// The person pressed Disconnect.
    Disconnect(BtAddress),
    /// The person pressed Connect on a nearby device: pair with it.
    Pair(BtAddress),
    /// The person pressed Forget.
    AskForget(BtAddress),
    /// The person confirmed forgetting.
    ConfirmForget,
    /// The person turned away from the question.
    Cancel,
    /// A device asks something.
    Asked(PairingQuestion),
    /// The person answered a pairing question.
    Answer(PairingAnswer),
    /// A connect, a pairing or a forget ended, with the words to say if it failed.
    Ended(BtAddress, Result<(), String>),
    /// BlueZ refused a request, in these words.
    Refused(String),
}

/// What the page asks of BlueZ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtEffect {
    /// Move the switch.
    SetPower(BtPower),
    /// Connect to a paired device.
    Connect(BtAddress),
    /// Disconnect from a device.
    Disconnect(BtAddress),
    /// Pair with a device.
    Pair(BtAddress),
    /// Remove a device.
    Forget(BtAddress),
    /// Answer the pairing question that waits.
    Answer(PairingAnswer),
    /// Say this to the person.
    Tell(String),
}

/// The next state, and what to ask of BlueZ.
pub fn bluetooth_step(state: BtState, input: BtInput) -> (BtState, Vec<BtEffect>) {
    match (state, input) {
        (state, BtInput::Heard(heard)) => (heard_step(state, heard), Vec::new()),
        (BtState::Ready(page), input) => {
            let (page, effects) = press_step(page, input);
            (BtState::Ready(page), effects)
        }
        (other, _) => (other, Vec::new()),
    }
}

fn heard_step(state: BtState, heard: Result<BtStatus, Absence>) -> BtState {
    let status = match heard {
        Ok(status) => status,
        Err(missing) => return BtState::Unavailable(missing),
    };
    match state {
        BtState::Ready(page) => BtState::Ready(BtReady { status, ..page }),
        BtState::Looking | BtState::Unavailable(_) => BtState::Ready(BtReady {
            status,
            asking: BtAsking::Nothing,
            busy: None,
        }),
    }
}

fn press_step(page: BtReady, input: BtInput) -> (BtReady, Vec<BtEffect>) {
    match input {
        BtInput::Heard(_) => (page, Vec::new()),
        BtInput::Switch(power) => (page, vec![BtEffect::SetPower(power)]),
        BtInput::Connect(address) => start(page, address, BtEffect::Connect),
        BtInput::Pair(address) => start(page, address, BtEffect::Pair),
        BtInput::Disconnect(address) => (page, vec![BtEffect::Disconnect(address)]),
        BtInput::AskForget(address) => (with_asking(page, BtAsking::Forget(address)), Vec::new()),
        BtInput::ConfirmForget => confirm_forget(page),
        BtInput::Cancel => cancel(page),
        BtInput::Asked(question) => (with_asking(page, BtAsking::Pairing(question)), Vec::new()),
        BtInput::Answer(answer) => (
            with_asking(page, BtAsking::Nothing),
            vec![BtEffect::Answer(answer)],
        ),
        BtInput::Ended(address, result) => ended(page, &address, result),
        BtInput::Refused(words) => (page, vec![BtEffect::Tell(words)]),
    }
}

fn with_asking(page: BtReady, asking: BtAsking) -> BtReady {
    BtReady { asking, ..page }
}

/// Starts work on a device, unless other work is under way.
fn start(
    page: BtReady,
    address: BtAddress,
    effect: fn(BtAddress) -> BtEffect,
) -> (BtReady, Vec<BtEffect>) {
    if page.busy.is_some() {
        return (page, Vec::new());
    }
    (
        BtReady {
            busy: Some(address.clone()),
            ..page
        },
        vec![effect(address)],
    )
}

fn confirm_forget(page: BtReady) -> (BtReady, Vec<BtEffect>) {
    match page.asking.clone() {
        BtAsking::Forget(address) => (
            with_asking(page, BtAsking::Nothing),
            vec![BtEffect::Forget(address)],
        ),
        BtAsking::Nothing | BtAsking::Pairing(_) => (page, Vec::new()),
    }
}

/// Turning away from a pairing question is a No to the device that asked.
fn cancel(page: BtReady) -> (BtReady, Vec<BtEffect>) {
    let effects = match &page.asking {
        BtAsking::Pairing(_) => vec![BtEffect::Answer(PairingAnswer::No)],
        BtAsking::Nothing | BtAsking::Forget(_) => Vec::new(),
    };
    (with_asking(page, BtAsking::Nothing), effects)
}

fn ended(
    page: BtReady,
    address: &BtAddress,
    result: Result<(), String>,
) -> (BtReady, Vec<BtEffect>) {
    let busy = page.busy.clone().filter(|busy| busy != address);
    let asking = match &page.asking {
        BtAsking::Pairing(question) if &question.address == address => BtAsking::Nothing,
        other => other.clone(),
    };
    let effects = result.err().map(BtEffect::Tell).into_iter().collect();
    (
        BtReady {
            busy,
            asking,
            ..page
        },
        effects,
    )
}

impl BtReady {
    /// Whether `address` is the device work is under way for, or is connecting.
    pub fn is_working(&self, address: &BtAddress) -> bool {
        self.busy.as_ref() == Some(address)
            || self
                .status
                .device(address)
                .is_some_and(|d| d.link == BtLink::Connecting)
    }

    /// Whether `address` is a device this computer is paired with.
    pub fn is_paired(&self, address: &BtAddress) -> bool {
        self.status
            .device(address)
            .is_some_and(|d| d.pairing == BtPairing::Paired)
    }
}

impl PairingQuestion {
    /// Whether the question wants an answer (the number to type on the device needs none).
    pub fn wants_answer(&self) -> bool {
        match self.ask {
            PairingAsk::Compare(_)
            | PairingAsk::Allow
            | PairingAsk::PinCode
            | PairingAsk::Passkey => true,
            PairingAsk::Show(_) => false,
        }
    }
}
