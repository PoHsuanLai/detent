//! The Software Update page: the service's words parsed, what an action may ask, and what each
//! standing shows. The words themselves are `detent-ui`'s copy and are tested there.

mod changed_offer;
mod parse;
mod press;
mod shown;

use super::{
    ByteCount, NotesState, Permille, ProblemKind, UnixSeconds, UpdateOffer, UpdatePhase,
    UpdateStatus, UpdateVersion, UpdatesNotice, UpdatesPage, UpdatesReady, UpdatesShown,
    UpdatesState, updates_shown,
};

pub(super) const NOW: UnixSeconds = UnixSeconds(1_000_000);

pub(super) fn version(text: &str) -> UpdateVersion {
    UpdateVersion::parse(text).expect("a version")
}

pub(super) fn offer() -> UpdateOffer {
    UpdateOffer {
        version: version("0.4.0"),
        notes: "Faster start.".to_owned(),
        size: ByteCount(45_600_000),
    }
}

pub(super) fn status(phase: UpdatePhase) -> UpdateStatus {
    UpdateStatus {
        phase,
        current: Some("0.3.1".to_owned()),
        offer: None,
        progress: Permille(0),
        last_checked: Some(UnixSeconds(NOW.0 - 300)),
        problem: None,
        problem_kind: ProblemKind::Other,
    }
}

pub(super) fn offered(phase: UpdatePhase) -> UpdateStatus {
    UpdateStatus {
        offer: Some(offer()),
        ..status(phase)
    }
}

pub(super) fn ready(status: UpdateStatus) -> UpdatesState {
    UpdatesState::Ready(UpdatesReady {
        status,
        info: NotesState::Closed,
        notice: UpdatesNotice::Nothing,
    })
}

pub(super) fn page_of(state: &UpdatesState) -> UpdatesPage {
    match updates_shown(state, NOW) {
        UpdatesShown::Page(page) => page,
        other => panic!("not a page: {other:?}"),
    }
}

/// A status whose offer is a newer one than [`offer`]'s.
pub(super) fn newer(phase: UpdatePhase) -> UpdateStatus {
    UpdateStatus {
        offer: Some(UpdateOffer {
            version: version("0.5.0"),
            ..offer()
        }),
        ..status(phase)
    }
}
