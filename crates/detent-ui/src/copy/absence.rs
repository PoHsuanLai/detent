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
    match subject {
        Subject::Accounts => accounts(absence),
        Subject::Runtimes => runtimes(absence),
        Subject::Computers => computers(absence),
        Subject::Models => models(absence),
        Subject::ScreenSharing => screen_sharing(absence),
        Subject::Spaces => spaces(absence),
        Subject::Privacy => privacy(absence),
        Subject::Wifi => system_page(absence, WIFI),
        Subject::Bluetooth => system_page(absence, BLUETOOTH),
        Subject::Displays => system_page(absence, DISPLAYS),
        Subject::Updates => updates(absence),
    }
}

fn accounts(absence: Absence) -> &'static str {
    match absence {
        Absence::NoBus => {
            "Settings is not connected to the desktop, so there are no accounts to show."
        }
        Absence::NotRunning => {
            "The accounts service is not running, so there are no accounts to show."
        }
        Absence::NotPermitted => {
            "Accounts can only be managed when Settings is opened from the desktop. Close this \
             window and open Settings again from the launcher."
        }
        Absence::Unavailable | Absence::NoDevice => {
            "The accounts service could not be confirmed, so there are no accounts to show."
        }
        Absence::Failed => "The accounts service did not answer. Try again in a moment.",
    }
}

fn runtimes(absence: Absence) -> &'static str {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop.",
        Absence::NotRunning => {
            "The accounts service is not running, so nothing on this computer can be listed."
        }
        Absence::NotPermitted => {
            "Models on this computer can only be listed when Settings is opened from the \
             desktop. Close this window and open Settings again from the launcher."
        }
        Absence::Unavailable | Absence::NoDevice => {
            "The accounts service could not be confirmed, so nothing on this computer can be \
             listed."
        }
        Absence::Failed => "The accounts service did not answer. Try again in a moment.",
    }
}

fn computers(absence: Absence) -> &'static str {
    match absence {
        Absence::NoBus => {
            "Settings is not connected to the desktop, so your computers cannot be listed."
        }
        Absence::NotRunning => "The assistant is not running, so your computers cannot be listed.",
        Absence::NotPermitted => {
            "Your computers can only be managed when Settings is opened from the desktop."
        }
        Absence::Unavailable | Absence::NoDevice => {
            "The assistant could not be confirmed, so your computers cannot be listed."
        }
        Absence::Failed => "The assistant did not answer. Try again in a moment.",
    }
}

fn models(absence: Absence) -> &'static str {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop.",
        Absence::NotRunning | Absence::Unavailable | Absence::NoDevice => {
            "Models are not available now."
        }
        Absence::NotPermitted => "Open Settings from the desktop to choose models.",
        Absence::Failed => "Models did not answer. Try again in a moment.",
    }
}

fn screen_sharing(absence: Absence) -> &'static str {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop, so there are no apps to show.",
        Absence::NotRunning => {
            "The screen sharing service could not be started, so there are no apps to show."
        }
        Absence::NotPermitted => {
            "Apps that may share your screen can only be managed when Settings is opened from the \
             desktop."
        }
        Absence::Unavailable | Absence::NoDevice => {
            "The screen sharing service could not be confirmed, so there are no apps to show."
        }
        Absence::Failed => "The screen sharing service did not answer. Try again in a moment.",
    }
}

fn spaces(absence: Absence) -> &'static str {
    match absence {
        Absence::NoBus => {
            "Settings is not connected to the desktop, so there are no Spaces to show."
        }
        Absence::NotRunning => {
            "The accounts service is not running, so there are no Spaces to show."
        }
        Absence::NotPermitted => {
            "Spaces can only be managed when Settings is opened from the desktop."
        }
        Absence::Unavailable | Absence::NoDevice => {
            "The accounts service could not be confirmed, so there are no Spaces to show."
        }
        Absence::Failed => "The accounts service did not answer. Try again in a moment.",
    }
}

fn privacy(absence: Absence) -> &'static str {
    match absence {
        Absence::NoBus => "Settings is not connected to the desktop.",
        Absence::NotRunning => "The desktop is not running.",
        Absence::NotPermitted => "Open Settings from the desktop to see Privacy.",
        Absence::Unavailable | Absence::NoDevice => "The desktop is not available now.",
        Absence::Failed => "The desktop did not answer. Try again in a moment.",
    }
}

/// What a page of the computer's own hardware says: when the service is not there, when there is
/// none of the hardware, and when it did not answer.
struct SystemPage {
    gone: &'static str,
    no_device: &'static str,
    failed: &'static str,
}

const WIFI: SystemPage = SystemPage {
    gone: "Wi-Fi isn\u{2019}t available right now.",
    no_device: "This computer has no Wi-Fi.",
    failed: "Wi-Fi didn\u{2019}t answer. Try again in a moment.",
};

const BLUETOOTH: SystemPage = SystemPage {
    gone: "Bluetooth isn\u{2019}t available right now.",
    no_device: "This computer has no Bluetooth.",
    failed: "Bluetooth didn\u{2019}t answer. Try again in a moment.",
};

const DISPLAYS: SystemPage = SystemPage {
    gone: "Displays aren\u{2019}t available right now.",
    no_device: "No display is connected.",
    failed: "The desktop didn\u{2019}t answer. Try again in a moment.",
};

fn system_page(absence: Absence, page: SystemPage) -> &'static str {
    match absence {
        Absence::NoBus | Absence::NotRunning | Absence::NotPermitted | Absence::Unavailable => {
            page.gone
        }
        Absence::NoDevice => page.no_device,
        Absence::Failed => page.failed,
    }
}

fn updates(absence: Absence) -> &'static str {
    match absence {
        Absence::Failed => "Software updates did not answer. Try again in a moment.",
        Absence::NoBus
        | Absence::NotRunning
        | Absence::NotPermitted
        | Absence::Unavailable
        | Absence::NoDevice => "Software updates are not available on this computer.",
    }
}

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
