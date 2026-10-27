//! The sharing switch's one transition: `(state, input) -> (state, effects)`.

/// The key of the switch "Let my other computers use this computer's models".
pub const SHARING_KEY: &str = "ai.tailnet.serve";

/// Which way the switch was turned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// To on.
    On,
    /// To off.
    Off,
}

/// What this computer already has for sharing, before the switch is turned on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Existing {
    /// Nothing.
    Absent,
    /// What Settings would set up.
    Same,
    /// Something else, which turning the switch on replaces.
    Different,
}

/// Where the switch stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sharing {
    /// Nothing is under way.
    #[default]
    Idle,
    /// The person turned it on and is being asked.
    Asking(Existing),
    /// The change is being made.
    Working(Way),
}

/// What happened to the switch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharingInput {
    /// The person turned it `Way`; `Existing` is what was there at that moment.
    Switched(Way, Existing),
    /// The person pressed Allow.
    Allowed,
    /// The person pressed Cancel.
    Cancelled,
    /// The change finished: done, or the sentence that says why not.
    Finished(Result<(), String>),
}

/// What the window does next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharingEffect {
    /// Set sharing up.
    Enable,
    /// Take sharing away.
    Disable,
    /// Write the setting.
    Record(Way),
    /// Tell the person.
    Tell(String),
}

/// The next state and what to do.
pub fn sharing_step(state: Sharing, input: SharingInput) -> (Sharing, Vec<SharingEffect>) {
    match (state, input) {
        (Sharing::Idle, SharingInput::Switched(Way::On, existing)) => {
            (Sharing::Asking(existing), Vec::new())
        }
        (Sharing::Idle, SharingInput::Switched(Way::Off, _)) => {
            (Sharing::Working(Way::Off), vec![SharingEffect::Disable])
        }
        (Sharing::Asking(_), SharingInput::Allowed) => {
            (Sharing::Working(Way::On), vec![SharingEffect::Enable])
        }
        (Sharing::Asking(_), SharingInput::Cancelled) => (Sharing::Idle, Vec::new()),
        (Sharing::Working(way), SharingInput::Finished(done)) => (
            Sharing::Idle,
            match (way, done) {
                (way, Ok(())) => vec![SharingEffect::Record(way)],
                (Way::On, Err(why)) => vec![SharingEffect::Tell(why)],
                // Off is recorded even when the file stayed: the service answers nobody while
                // the setting is off.
                (Way::Off, Err(why)) => {
                    vec![SharingEffect::Record(Way::Off), SharingEffect::Tell(why)]
                }
            },
        ),
        (state, _) => (state, Vec::new()),
    }
}
