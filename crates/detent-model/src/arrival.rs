//! Where a [`SettingsTarget`] lands in the window: the page, with the search that marks the row,
//! or the Accounts page at an account. Pure: it only reads the catalogue.

use crate::accounts::{AccountSegment, AccountTarget};
use crate::assistant_model::is_assistant_setup;
use crate::catalogue::Catalogue;
use crate::page::PageSlug;
use ds_settings::target::SettingsTarget;
use porter_core::{AccountId, object_segment};

/// What the window does for a target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arrival {
    /// Show this page; `query` is what the search box then holds (the key's label, so the
    /// search highlights the row and opens the advanced list when the key is there).
    Page { slug: PageSlug, query: String },
    /// Show the Accounts page as the target asks.
    Account(AccountTarget),
}

/// The account a target's id names, by the segment accountd's object path ends in; an id that is
/// not an account's gives the list.
fn account_of(id: &str) -> AccountTarget {
    AccountId::parse(id).map_or(AccountTarget::List, |id| {
        AccountTarget::Detail(AccountSegment(object_segment(&id)))
    })
}

/// Where `target` lands, or none when no page of the window is what it names.
pub fn arrival(catalogue: &Catalogue, target: &SettingsTarget) -> Option<Arrival> {
    match target {
        SettingsTarget::Page(id) => {
            let slug = PageSlug(id.as_str().to_owned());
            (catalogue.page(&slug).is_some() || is_assistant_setup(&slug)).then(|| Arrival::Page {
                slug,
                query: String::new(),
            })
        }
        SettingsTarget::Key(path) => catalogue
            .entries()
            .find(|(_, entry)| &entry.spec.path == path)
            .map(|(page, entry)| Arrival::Page {
                slug: page.slug.clone(),
                query: entry.spec.label.0.clone(),
            }),
        SettingsTarget::Account(account) => Some(Arrival::Account(account_of(&account.0))),
    }
}

#[cfg(test)]
mod tests {
    use super::{Arrival, arrival};
    use crate::accounts::{AccountSegment, AccountTarget};
    use crate::catalogue::Catalogue;
    use crate::page::PageSlug;
    use ds_settings::target::SettingsTarget;

    fn target(text: &str) -> SettingsTarget {
        text.parse().expect("a target")
    }

    #[test]
    fn a_page_the_catalogue_lacks_lands_nowhere() {
        assert_eq!(arrival(&Catalogue::default(), &target("page:dock")), None);
    }

    #[test]
    fn a_key_the_catalogue_lacks_lands_nowhere() {
        assert_eq!(
            arrival(&Catalogue::default(), &target("key:dock.size")),
            None
        );
    }

    #[test]
    fn the_setup_step_is_a_page() {
        assert_eq!(
            arrival(&Catalogue::default(), &target("page:assistant-model-setup")),
            Some(Arrival::Page {
                slug: PageSlug("assistant-model-setup".to_owned()),
                query: String::new(),
            })
        );
    }

    #[test]
    fn an_account_lands_on_its_segment() {
        assert_eq!(
            arrival(&Catalogue::default(), &target("account:work-mail")),
            Some(Arrival::Account(AccountTarget::Detail(AccountSegment(
                "work_mail".to_owned()
            ))))
        );
    }

    #[test]
    fn an_id_that_is_no_account_lands_on_the_list() {
        assert_eq!(
            arrival(&Catalogue::default(), &target("account:Not_An_Id")),
            Some(Arrival::Account(AccountTarget::List))
        );
    }
}
