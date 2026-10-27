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
    /// Found and not usable, with why.
    Unusable(ExtensionTrouble),
}

/// Why an Extension cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionTrouble {
    /// Its description could not be read at all.
    Unreadable,
    /// The Extensions library found something wrong, and said it in its own sentence.
    Reported(String),
}

/// The kinds of group the page lists, in the order they come.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionKind {
    /// Extensions that provide a look.
    Looks,
    /// Extensions that provide file icons.
    FileIcons,
    /// Extensions that provide none of those.
    Other,
    /// Extensions that cannot be used, last.
    Unusable,
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
const GROUPS: &[(&str, ExtensionKind)] = &[
    ("look", ExtensionKind::Looks),
    ("file_icons", ExtensionKind::FileIcons),
];

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

    fn kind(&self) -> ExtensionKind {
        if matches!(self.standing, ExtensionStanding::Unusable(_)) {
            return ExtensionKind::Unusable;
        }
        GROUPS
            .iter()
            .find(|(word, _)| self.provides.iter().any(|provided| provided == word))
            .map_or(ExtensionKind::Other, |(_, kind)| *kind)
    }
}

/// Extensions that provide the same kind of thing, or that cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionGroup {
    /// What its Extensions have in common.
    pub kind: ExtensionKind,
    /// Its rows, by name.
    pub rows: Vec<ExtensionRow>,
}

/// The page's groups, empty ones left out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtensionList {
    /// In the order of [`ExtensionKind`].
    pub groups: Vec<ExtensionGroup>,
}

impl ExtensionList {
    /// The list of `rows`, grouped and sorted by name.
    pub fn of(rows: Vec<ExtensionRow>) -> ExtensionList {
        let order = GROUPS
            .iter()
            .map(|(_, kind)| *kind)
            .chain([ExtensionKind::Other, ExtensionKind::Unusable]);
        let groups = order
            .filter_map(|kind| {
                let mut rows: Vec<ExtensionRow> = rows
                    .iter()
                    .filter(|row| row.kind() == kind)
                    .cloned()
                    .collect();
                rows.sort_by_cached_key(|row| (row.name.to_lowercase(), row.id.clone()));
                (!rows.is_empty()).then_some(ExtensionGroup { kind, rows })
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
    use super::{
        ExtensionKeep, ExtensionKind, ExtensionList, ExtensionPower, ExtensionRow,
        ExtensionStanding, ExtensionTrouble,
    };

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
        let broken = ExtensionStanding::Unusable(ExtensionTrouble::Unreadable);
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
        let shape: Vec<(ExtensionKind, Vec<&str>)> = list
            .groups
            .iter()
            .map(|group| {
                (
                    group.kind,
                    group.rows.iter().map(|row| row.id.as_str()).collect(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            [
                (ExtensionKind::Looks, vec!["e.both", "b.zed"]),
                (ExtensionKind::FileIcons, vec!["a.icons"]),
                (ExtensionKind::Other, vec!["d.other"]),
                (ExtensionKind::Unusable, vec!["c.broken"]),
            ]
        );
    }

    #[test]
    fn only_a_usable_extension_has_a_switch() {
        let mut usable = row("a.b", "Paper", &["look"], ExtensionStanding::Off);
        usable.summary = Some("Warm paper".to_owned());
        assert_eq!(usable.power(), Some(ExtensionPower::Off));
        let mut broken = row(
            "a.c",
            "Torn",
            &["look"],
            ExtensionStanding::Unusable(ExtensionTrouble::Reported("Torn".to_owned())),
        );
        broken.summary = Some("Warm paper".to_owned());
        assert_eq!(broken.power(), None);
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
