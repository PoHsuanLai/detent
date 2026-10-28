//! What a pane says when the service behind it is not there: one table, a row per pane, a column
//! per [`Absence`].

use detent_model::Absence;

/// The pane that has nothing to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subject {
    /// The Accounts pane.
    Accounts,
    /// "On this computer": the local runtimes.
    Runtimes,
    /// "Your computers".
    Computers,
    /// The model picker.
    Models,
    /// The apps that may share the screen.
    ScreenSharing,
    /// The desktop-wide Spaces.
    Spaces,
    /// The Privacy page's compositor rows.
    Privacy,
    /// The Wi-Fi page.
    Wifi,
    /// The Bluetooth page.
    Bluetooth,
    /// The Displays page.
    Displays,
    /// The Software Update page.
    Updates,
}

/// What `subject` says when `absence` is why it has nothing to show.
pub(crate) fn absence(subject: Subject, absence: Absence) -> &'static str {
    let words = match subject {
        Subject::Accounts => &ACCOUNTS,
        Subject::Runtimes => &RUNTIMES,
        Subject::Computers => &COMPUTERS,
        Subject::Models => &MODELS,
        Subject::ScreenSharing => &SCREEN_SHARING,
        Subject::Spaces => &SPACES,
        Subject::Privacy => &PRIVACY,
        Subject::Wifi => &WIFI,
        Subject::Bluetooth => &BLUETOOTH,
        Subject::Displays => &DISPLAYS,
        Subject::Updates => &UPDATES,
    };
    match absence {
        Absence::NoBus => words.no_bus,
        Absence::NotRunning => words.not_running,
        Absence::NotPermitted => words.not_permitted,
        Absence::Unavailable => words.unavailable,
        Absence::NoDevice => words.no_device,
        Absence::Failed => words.failed,
    }
}

/// What one pane says for each [`Absence`].
struct Words {
    no_bus: &'static str,
    not_running: &'static str,
    not_permitted: &'static str,
    unavailable: &'static str,
    no_device: &'static str,
    failed: &'static str,
}

impl Words {
    /// A page of the computer's own hardware: one sentence when the service is not there, one when
    /// there is none of the hardware, and one when it did not answer.
    const fn system_page(
        gone: &'static str,
        no_device: &'static str,
        failed: &'static str,
    ) -> Self {
        Self {
            no_bus: gone,
            not_running: gone,
            not_permitted: gone,
            unavailable: gone,
            no_device,
            failed,
        }
    }
}

const NOT_CONNECTED: &str = "Settings is not connected to the desktop.";

const ACCOUNTS: Words = Words {
    no_bus: "Settings is not connected to the desktop, so there are no accounts to show.",
    not_running: "The accounts service is not running, so there are no accounts to show.",
    not_permitted: "Accounts can only be managed when Settings is opened from the desktop. Close \
                    this window and open Settings again from the launcher.",
    unavailable: "The accounts service could not be confirmed, so there are no accounts to show.",
    no_device: "The accounts service could not be confirmed, so there are no accounts to show.",
    failed: "The accounts service did not answer. Try again in a moment.",
};

const RUNTIMES: Words = Words {
    no_bus: NOT_CONNECTED,
    not_running: "The accounts service is not running, so nothing on this computer can be listed.",
    not_permitted: "Models on this computer can only be listed when Settings is opened from the \
                    desktop. Close this window and open Settings again from the launcher.",
    unavailable: "The accounts service could not be confirmed, so nothing on this computer can be \
                  listed.",
    no_device: "The accounts service could not be confirmed, so nothing on this computer can be \
                listed.",
    failed: "The accounts service did not answer. Try again in a moment.",
};

const COMPUTERS: Words = Words {
    no_bus: "Settings is not connected to the desktop, so your computers cannot be listed.",
    not_running: "The assistant is not running, so your computers cannot be listed.",
    not_permitted: "Your computers can only be managed when Settings is opened from the desktop.",
    unavailable: "The assistant could not be confirmed, so your computers cannot be listed.",
    no_device: "The assistant could not be confirmed, so your computers cannot be listed.",
    failed: "The assistant did not answer. Try again in a moment.",
};

const MODELS: Words = Words {
    no_bus: NOT_CONNECTED,
    not_running: "Models are not available now.",
    not_permitted: "Open Settings from the desktop to choose models.",
    unavailable: "Models are not available now.",
    no_device: "Models are not available now.",
    failed: "Models did not answer. Try again in a moment.",
};

const SCREEN_SHARING: Words = Words {
    no_bus: "Settings is not connected to the desktop, so there are no apps to show.",
    not_running: "The screen sharing service could not be started, so there are no apps to show.",
    not_permitted: "Apps that may share your screen can only be managed when Settings is opened \
                    from the desktop.",
    unavailable: "The screen sharing service could not be confirmed, so there are no apps to show.",
    no_device: "The screen sharing service could not be confirmed, so there are no apps to show.",
    failed: "The screen sharing service did not answer. Try again in a moment.",
};

const SPACES: Words = Words {
    no_bus: "Settings is not connected to the desktop, so there are no Spaces to show.",
    not_running: "The accounts service is not running, so there are no Spaces to show.",
    not_permitted: "Spaces can only be managed when Settings is opened from the desktop.",
    unavailable: "The accounts service could not be confirmed, so there are no Spaces to show.",
    no_device: "The accounts service could not be confirmed, so there are no Spaces to show.",
    failed: "The accounts service did not answer. Try again in a moment.",
};

const PRIVACY: Words = Words {
    no_bus: NOT_CONNECTED,
    not_running: "The desktop is not running.",
    not_permitted: "Open Settings from the desktop to see Privacy.",
    unavailable: "The desktop is not available now.",
    no_device: "The desktop is not available now.",
    failed: "The desktop did not answer. Try again in a moment.",
};

const WIFI: Words = Words::system_page(
    "Wi-Fi isn\u{2019}t available right now.",
    "This computer has no Wi-Fi.",
    "Wi-Fi didn\u{2019}t answer. Try again in a moment.",
);

const BLUETOOTH: Words = Words::system_page(
    "Bluetooth isn\u{2019}t available right now.",
    "This computer has no Bluetooth.",
    "Bluetooth didn\u{2019}t answer. Try again in a moment.",
);

const DISPLAYS: Words = Words::system_page(
    "Displays aren\u{2019}t available right now.",
    "No display is connected.",
    "The desktop didn\u{2019}t answer. Try again in a moment.",
);

const UPDATES: Words = Words::system_page(
    "Software updates are not available on this computer.",
    "Software updates are not available on this computer.",
    "Software updates did not answer. Try again in a moment.",
);

#[cfg(test)]
mod tests {
    use super::{Subject, absence};
    use detent_model::Absence;

    #[test]
    fn every_pane_says_something_for_every_absence() {
        const ALL: [Absence; 6] = [
            Absence::NoBus,
            Absence::NotRunning,
            Absence::NotPermitted,
            Absence::Unavailable,
            Absence::NoDevice,
            Absence::Failed,
        ];
        const SUBJECTS: [Subject; 11] = [
            Subject::Accounts,
            Subject::Runtimes,
            Subject::Computers,
            Subject::Models,
            Subject::ScreenSharing,
            Subject::Spaces,
            Subject::Privacy,
            Subject::Wifi,
            Subject::Bluetooth,
            Subject::Displays,
            Subject::Updates,
        ];
        for subject in SUBJECTS {
            for why in ALL {
                assert!(!absence(subject, why).is_empty(), "{subject:?} {why:?}");
            }
        }
    }

    #[test]
    fn the_words_are_the_ones_each_pane_always_had() {
        // `(pane, why, what is read)`.
        const ROWS: &[(Subject, Absence, &str)] = &[
            (
                Subject::Accounts,
                Absence::NotRunning,
                "The accounts service is not running, so there are no accounts to show.",
            ),
            (
                Subject::Models,
                Absence::NotPermitted,
                "Open Settings from the desktop to choose models.",
            ),
            (
                Subject::Wifi,
                Absence::NotRunning,
                "Wi-Fi isn\u{2019}t available right now.",
            ),
            (
                Subject::Wifi,
                Absence::NoDevice,
                "This computer has no Wi-Fi.",
            ),
            (
                Subject::Bluetooth,
                Absence::Failed,
                "Bluetooth didn\u{2019}t answer. Try again in a moment.",
            ),
            (
                Subject::Displays,
                Absence::NoDevice,
                "No display is connected.",
            ),
            (
                Subject::Updates,
                Absence::Failed,
                "Software updates did not answer. Try again in a moment.",
            ),
            (
                Subject::Updates,
                Absence::NoBus,
                "Software updates are not available on this computer.",
            ),
            (
                Subject::Privacy,
                Absence::NotRunning,
                "The desktop is not running.",
            ),
        ];
        for (subject, why, want) in ROWS {
            assert_eq!(absence(*subject, *why), *want, "{subject:?} {why:?}");
        }
    }
}
