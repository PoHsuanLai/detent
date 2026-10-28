//! An account's avatar: its provider's mark, or a lettered disc when the module names no
//! provider; a ring turns round it while the sync service is moving data for the account.

use crate::marks::ProviderMarks;
use detent_model::AccountProvider;
use dioxus::prelude::*;
use ds::components::content::avatar::{AvatarSize, AvatarTone, person_hue};
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;

/// Where the avatar is drawn: it is larger at the head of an account's detail than in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AvatarPlace {
    /// A row of the list.
    Row,
    /// The head of the account's detail.
    Header,
}

impl AvatarPlace {
    fn mark_size(self) -> ControlSize {
        match self {
            AvatarPlace::Row => ControlSize::Regular,
            AvatarPlace::Header => ControlSize::Large,
        }
    }

    fn disc_size(self) -> AvatarSize {
        match self {
            AvatarPlace::Row => AvatarSize::Size28,
            AvatarPlace::Header => AvatarSize::Size48,
        }
    }
}

/// The mark of `provider` (or the disc of `label`'s first letter), inside the sync ring while
/// `work` is `Working`.
#[component]
pub(super) fn AccountAvatar(
    label: String,
    provider: Option<AccountProvider>,
    marks: ProviderMarks,
    place: AvatarPlace,
    syncing: bool,
) -> Element {
    let face = match provider {
        Some(provider) => {
            let mark = marks.of(&provider.id);
            rsx! {
                ProviderMark { provider: mark.provider, face: mark.face, size: place.mark_size() }
            }
        }
        None => rsx! {
            Avatar {
                initial: label.chars().next().unwrap_or('?').to_ascii_uppercase(),
                size: place.disc_size(),
                tone: AvatarTone::Person(person_hue(&label)),
            }
        },
    };
    if syncing {
        rsx! {
            Halo { centre: face, label }
        }
    } else {
        face
    }
}

/// The ring that turns round an avatar while its account syncs.
#[component]
fn Halo(centre: Element, label: String) -> Element {
    let token = use_hook(PendingToken::start);
    rsx! {
        ProgressIndicator {
            style: ProgressStyle::Ring,
            progress: Progress::Unknown(Operation::Running(token)),
            size: ControlSize::Large,
            centre: Some(centre),
            common: Common { aria_label: Some(crate::copy::account_views::syncing(&label)), ..Common::default() },
        }
    }
}
