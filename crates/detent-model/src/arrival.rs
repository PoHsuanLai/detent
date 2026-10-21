//! Where a [`SettingsTarget`] lands in the window: the page, with the search that marks the row,
//! or the Accounts page at an account. Pure: it only reads the catalogue.

use crate::accounts::{AccountSegment, AccountTarget, is_accounts_setup};
use crate::assistant_model::is_assistant_setup;
use crate::catalogue::Catalogue;
use crate::page::PageSlug;
use ds_settings::schema::PageTable;
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

/// The page `text` names, through quire's page table (an id, an alias, the old `app:` form): the
/// catalogue's page that the table declares under that id, else the hidden first-run step (which
/// the table resolves but the sidebar does not list), else the accounts step (which only detent
/// declares), else a page only the catalogue knows by that slug.
fn page_slug(catalogue: &Catalogue, text: &str) -> Option<PageSlug> {
    let table = PageTable::built_in();
    let declared = table.resolve(text);
    declared
        .as_ref()
        .and_then(|id| {
            catalogue.pages.iter().find(|model| {
                table
                    .declared(&model.page)
                    .is_some_and(|decl| decl.id == *id)
            })
        })
        .or_else(|| catalogue.page(&PageSlug(text.to_owned())))
        .map(|model| model.slug.clone())
        .or_else(|| declared.map(|id| PageSlug(id.0)).filter(is_assistant_setup))
        .or_else(|| Some(PageSlug(text.to_owned())).filter(is_accounts_setup))
}

/// Where `target` lands, or none when no page of the window is what it names.
pub fn arrival(catalogue: &Catalogue, target: &SettingsTarget) -> Option<Arrival> {
    match target {
        SettingsTarget::Page(id) => page_slug(catalogue, id.as_str()).map(|slug| Arrival::Page {
            slug,
            query: String::new(),
        }),
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
    fn the_setup_steps_are_pages() {
        for slug in ["assistant-model-setup", "accounts-setup"] {
            assert_eq!(
                arrival(&Catalogue::default(), &target(&format!("page:{slug}"))),
                Some(Arrival::Page {
                    slug: PageSlug(slug.to_owned()),
                    query: String::new(),
                }),
                "{slug}"
            );
        }
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
