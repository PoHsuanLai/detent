//! The person at the top of the sidebar: their name and picture.

use std::path::PathBuf;

/// What the sidebar's card under the search field says of the person using this computer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    /// Their full name, or their login name when no full name is kept.
    pub name: String,
    /// Their picture, a file, when they have one.
    pub picture: Option<PathBuf>,
}

impl Person {
    /// The first letter of the name, upper case, for a card that has no picture.
    pub fn initial(&self) -> char {
        self.name
            .chars()
            .find(|c| c.is_alphanumeric())
            .map_or('?', |c| c.to_uppercase().next().unwrap_or(c))
    }
}

/// The person `uid` is in the text of a passwd file: the first part of the comment field, else the
/// login name. None when no line is for `uid`.
pub fn passwd_name(passwd: &str, uid: u32) -> Option<String> {
    passwd.lines().find_map(|line| {
        let mut fields = line.split(':');
        let login = fields.next()?;
        let line_uid: u32 = fields.nth(1)?.parse().ok()?;
        if line_uid != uid {
            return None;
        }
        let comment = fields.nth(1).unwrap_or_default();
        let full = comment.split(',').next().unwrap_or_default().trim();
        let name = if full.is_empty() { login } else { full };
        Some(name.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::{Person, passwd_name};

    const PASSWD: &str = "root:x:0:0:root:/root:/bin/bash\n\
        ada:x:1000:1000:Ada Lovelace,,,:/home/ada:/bin/bash\n\
        svc:x:1001:1001::/home/svc:/bin/false\n";

    #[test]
    fn a_name_comes_from_the_comment_field_else_the_login() {
        const CASES: &[(u32, Option<&str>)] = &[
            (1000, Some("Ada Lovelace")),
            (1001, Some("svc")),
            (0, Some("root")),
            (42, None),
        ];
        for (uid, want) in CASES {
            assert_eq!(passwd_name(PASSWD, *uid).as_deref(), *want, "uid {uid}");
        }
    }

    #[test]
    fn the_initial_is_the_first_letter_in_upper_case() {
        for (name, want) in [("ada lovelace", 'A'), ("  \u{e9}mile", '\u{c9}'), ("", '?')] {
            let person = Person {
                name: name.to_owned(),
                picture: None,
            };
            assert_eq!(person.initial(), want, "{name:?}");
        }
    }
}
