//! The marks of the providers Settings shows: each shipped provider file names a mark and may give
//! it a face (a letter on a colour). Quire draws the face first, then the named mark, then the
//! neutral `@`.

use ds::components::content::mark_face::MarkFace;
use ds::components::content::provider_mark::MarkProvider;
use porter_provider::shipped_specs;
use std::collections::BTreeMap;
use std::sync::Arc;

/// What a provider's mark is drawn from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkSource {
    /// The named mark; the neutral one for a provider quire has no name for.
    pub provider: MarkProvider,
    /// The letter and colour the provider's file gives, if it does.
    pub face: Option<MarkFace>,
}

/// The mark of each shipped provider, by the provider's id.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProviderMarks(Arc<BTreeMap<String, MarkSource>>);

/// The named mark a provider file's `mark` word stands for.
fn named(word: &str) -> MarkProvider {
    match word {
        "google" => MarkProvider::Google,
        "microsoft" => MarkProvider::Microsoft,
        "fastmail" => MarkProvider::Fastmail,
        "icloud" => MarkProvider::ICloud,
        "yahoo" => MarkProvider::Yahoo,
        "local" => MarkProvider::Local,
        _ => MarkProvider::Imap,
    }
}

impl ProviderMarks {
    /// The marks of the provider files porter ships.
    pub fn shipped() -> ProviderMarks {
        let marks = shipped_specs()
            .into_iter()
            .map(|spec| {
                let face = spec
                    .mark_face
                    .as_ref()
                    .map(|face| MarkFace::new(face.letter.as_str(), face.colour.as_str()));
                let source = MarkSource {
                    provider: named(&spec.mark),
                    face,
                };
                (spec.id.as_str().to_owned(), source)
            })
            .collect();
        ProviderMarks(Arc::new(marks))
    }

    /// What to draw for provider `id`: the neutral mark for one whose file is not here.
    pub(crate) fn of(&self, id: &str) -> MarkSource {
        self.0.get(id).cloned().unwrap_or(MarkSource {
            provider: MarkProvider::Imap,
            face: None,
        })
    }
}
