//! What the provider says back: refusals in words, outcomes, labels and entity references.

use detent_model::KeyEntry;
use docket_core::{AppRefusal, FailText, Follow, Outcome, Preview, Undoable};
use docket_vocab::{EntityRef, LabelText};
use porter_core::AppName;
use prov::{Confidentiality, EntityId, EntityKey, EntityKind, Integrity, Label, Labelled, Source};
use std::collections::BTreeSet;

/// Words an app wrote itself, trusted: ids, labels and counts.
pub(crate) fn own_label(app: &AppName) -> Label {
    Label {
        integrity: Integrity::Trusted,
        confidentiality: Confidentiality::Public,
        classes: BTreeSet::new(),
        sources: BTreeSet::from([Source::App(app.clone())]),
    }
}

/// The app's own words, labelled.
pub(crate) fn own(app: &AppName, text: impl Into<String>) -> Labelled<String> {
    Labelled::new(text.into(), own_label(app))
}

/// A refusal that says why.
pub(crate) fn refusal(why: impl Into<String>) -> AppRefusal {
    AppRefusal::Failed(FailText(why.into()))
}

/// The entity `kind` named by `key`, or `None` if either is malformed.
pub(crate) fn entity(app: &AppName, kind: &str, key: &str) -> Option<EntityId> {
    Some(EntityId {
        app: app.clone(),
        kind: EntityKind::parse(kind).ok()?,
        key: EntityKey::parse(key).ok()?,
    })
}

/// The refusal for a thing that is not there.
pub(crate) fn not_found(app: &AppName, kind: &str, key: &str) -> AppRefusal {
    entity(app, kind, key).map_or(AppRefusal::Unsupported, AppRefusal::NotFound)
}

/// A key as a thing a person or an agent can name.
pub(crate) fn key_ref(app: &AppName, entry: &KeyEntry) -> Option<EntityRef> {
    Some(EntityRef {
        id: entity(app, "settings.key", &entry.spec.path.0)?,
        title: own(app, entry.spec.label.0.clone()),
        subtitle: own(app, entry.spec.path.0.clone()),
    })
}

/// A page as a thing a person or an agent can name.
pub(crate) fn page_ref(app: &AppName, slug: &str, title: &str) -> Option<EntityRef> {
    Some(EntityRef {
        id: entity(app, "settings.page", slug)?,
        title: own(app, title),
        subtitle: own(app, slug),
    })
}

/// A finished action: what to say (dropped when it is not valid UI words), what can undo it.
pub(crate) fn outcome(said: &str, undo: Undoable) -> Outcome {
    Outcome {
        value: None,
        said: LabelText::parse(said).ok(),
        show: Preview::None,
        undo,
        follow: Follow::Nothing,
    }
}
