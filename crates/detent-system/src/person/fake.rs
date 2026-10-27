//! A person from memory.

use super::PersonLink;
use detent_model::{Person, Work};

/// A person that answers with what it was given.
#[derive(Debug, Clone)]
pub struct FakePerson(Person);

impl FakePerson {
    /// This person, with no picture.
    pub fn named(name: &str) -> FakePerson {
        FakePerson(Person {
            name: name.to_owned(),
            picture: None,
        })
    }

    /// This person, with the picture at `picture`.
    #[must_use]
    pub fn with_picture(self, picture: std::path::PathBuf) -> FakePerson {
        FakePerson(Person {
            picture: Some(picture),
            ..self.0
        })
    }
}

impl PersonLink for FakePerson {
    fn who(&self) -> Work<'_, Option<Person>> {
        Box::pin(std::future::ready(Some(self.0.clone())))
    }
}
