//! The person at the top of the sidebar: who is using this computer.

use detent_model::Person;
use detent_system::person::PersonLink;
use std::sync::Arc;

/// The person's handle: cheap to clone.
#[derive(Debug, Clone)]
pub struct Whoami(Arc<dyn PersonLink>);

impl Whoami {
    pub(crate) fn over(link: Arc<dyn PersonLink>) -> Whoami {
        Whoami(link)
    }

    /// Their name and picture, or none when nothing says who they are.
    pub async fn person(&self) -> Option<Person> {
        self.0.who().await
    }
}
