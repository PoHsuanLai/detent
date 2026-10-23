//! The installed Extensions as the Extensions page lists them: grouped by what they provide,
//! each with the switch that is its standing.

use super::offers::Offers;

/// Whether an Extension is in use. `On` is the default for every Extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionPower {
    /// In use.
    On,
    /// Turned off by the person: it stays installed and does nothing.
    Off,
}

/// Where an Extension stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionStanding {
    /// In use.
    On,
    /// Turned off.
    Off,
    /// Found and not usable, with why in plain words.
    Unusable(String),
}

/// Whether an Extension may be switched off (and so taken away): the one that holds the
/// person's own changes may not while it holds any, since that would throw them away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtensionKeep {
    /// Nothing in it is the person's own: the switch is offered.
    #[default]
    Free,
    /// It holds changes of the person's: no switch.
    Holds,
}

/// One installed Extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionRow {
    /// Its id, the name a link to it carries.
    pub id: String,
    /// What a person calls it.
    pub name: String,
    /// The one line it says of itself.
    pub summary: Option<String>,
    /// The capability words it provides (`look`).
    pub provides: Vec<String>,
    /// Where it stands.
    pub standing: ExtensionStanding,
    /// Whether the switch is offered.
    pub keep: ExtensionKeep,
}

/// The capability words that have a group of their own, in the order the groups come.
const GROUPS: &[(&str, &str)] = &[("look", "Looks"), ("file_icons", "File Icons")];

/// The group of Extensions that provide none of those words.
const OTHER: &str = "Other";

/// The group of Extensions that cannot be used, last.
const UNUSABLE: &str = "Can\u{2019}t Be Used";

impl ExtensionRow {
    /// The switch this row draws: none for an Extension that cannot be used.
    pub fn power(&self) -> Option<ExtensionPower> {
        match (&self.standing, self.keep) {
            (_, ExtensionKeep::Holds) | (ExtensionStanding::Unusable(_), _) => None,
            (ExtensionStanding::On, ExtensionKeep::Free) => Some(ExtensionPower::On),
            (ExtensionStanding::Off, ExtensionKeep::Free) => Some(ExtensionPower::Off),
        }
    }

    /// Whether the Extension cannot be used (the page says so where the switch would be).
    pub fn is_unusable(&self) -> bool {
        matches!(self.standing, ExtensionStanding::Unusable(_))
    }

    /// The line under the name: why it cannot be used, else what it says of itself.
    pub fn line(&self) -> Option<&str> {
        match &self.standing {
            ExtensionStanding::Unusable(reason) => Some(reason),
            ExtensionStanding::On | ExtensionStanding::Off => self.summary.as_deref(),
        }
    }

    fn group(&self) -> &'static str {
        if matches!(self.standing, ExtensionStanding::Unusable(_)) {
            return UNUSABLE;
        }
        GROUPS
            .iter()
            .find(|(word, _)| self.provides.iter().any(|provided| provided == word))
            .map_or(OTHER, |(_, title)| *title)
    }
}

/// Extensions that provide the same kind of thing, or that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionGroup {
    /// The heading over the card.
    pub title: String,
    /// Its rows, by name.
    pub rows: Vec<ExtensionRow>,
}

/// The page's groups, empty ones left out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtensionList {
    /// In the order Looks, File Icons, Other, Can't Be Used.
    pub groups: Vec<ExtensionGroup>,
}

impl ExtensionList {
    /// The list of `rows`, grouped and sorted by name.
    pub fn of(rows: Vec<ExtensionRow>) -> ExtensionList {
        let order = GROUPS
            .iter()
            .map(|(_, title)| *title)
            .chain([OTHER, UNUSABLE]);
        let groups = order
            .filter_map(|title| {
                let mut rows: Vec<ExtensionRow> = rows
                    .iter()
                    .filter(|row| row.group() == title)
                    .cloned()
                    .collect();
                rows.sort_by_cached_key(|row| (row.name.to_lowercase(), row.id.clone()));
                (!rows.is_empty()).then(|| ExtensionGroup {
                    title: title.to_owned(),
                    rows,
                })
            })
            .collect();
        ExtensionList { groups }
    }

    /// Whether nothing is installed.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

/// Everything the window reads of the installed Extensions in one look at the folders.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstalledExtensions {
    /// What the `Provided` keys offer.
    pub offers: Offers,
    /// What the Extensions page lists.
    pub list: ExtensionList,
}

#[cfg(test)]
mod tests {
    use super::{ExtensionKeep, ExtensionList, ExtensionPower, ExtensionRow, ExtensionStanding};

    fn row(id: &str, name: &str, provides: &[&str], standing: ExtensionStanding) -> ExtensionRow {
        ExtensionRow {
            id: id.to_owned(),
            name: name.to_owned(),
            summary: None,
            provides: provides.iter().map(|word| (*word).to_owned()).collect(),
            standing,
            keep: ExtensionKeep::Free,
        }
    }

    #[test]
    fn rows_are_grouped_by_what_they_provide_then_the_unusable_last() {
        let broken = ExtensionStanding::Unusable("Some files could not be read".to_owned());
        let list = ExtensionList::of(vec![
            row("c.broken", "Broken", &["look"], broken),
            row("b.zed", "Zed", &["look"], ExtensionStanding::On),
            row("a.icons", "Icons", &["file_icons"], ExtensionStanding::Off),
            row("d.other", "Thumbs", &["thumbnail"], ExtensionStanding::On),
            row(
                "e.both",
                "Alpha",
                &["file_icons", "look"],
                ExtensionStanding::On,
            ),
        ]);
        let shape: Vec<(&str, Vec<&str>)> = list
            .groups
            .iter()
            .map(|group| {
                (
                    group.title.as_str(),
                    group.rows.iter().map(|row| row.id.as_str()).collect(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            [
                ("Looks", vec!["e.both", "b.zed"]),
                ("File Icons", vec!["a.icons"]),
                ("Other", vec!["d.other"]),
                ("Can\u{2019}t Be Used", vec!["c.broken"]),
            ]
        );
    }

    #[test]
    fn only_a_usable_extension_has_a_switch_and_the_reason_replaces_its_summary() {
        let mut usable = row("a.b", "Paper", &["look"], ExtensionStanding::Off);
        usable.summary = Some("Warm paper".to_owned());
        assert_eq!(usable.power(), Some(ExtensionPower::Off));
        assert_eq!(usable.line(), Some("Warm paper"));
        let mut broken = row(
            "a.c",
            "Torn",
            &["look"],
            ExtensionStanding::Unusable("Torn".to_owned()),
        );
        broken.summary = Some("Warm paper".to_owned());
        assert_eq!(broken.power(), None);
        assert_eq!(broken.line(), Some("Torn"));
        assert!(ExtensionList::of(Vec::new()).is_empty());
    }

    #[test]
    fn an_extension_holding_the_persons_changes_has_no_switch_and_is_still_usable() {
        let mut mine = row(
            "user.YourChanges",
            "Your changes",
            &["file_icons"],
            ExtensionStanding::On,
        );
        assert_eq!(mine.power(), Some(ExtensionPower::On));
        mine.keep = ExtensionKeep::Holds;
        assert_eq!(mine.power(), None);
        assert!(!mine.is_unusable());
    }
}
