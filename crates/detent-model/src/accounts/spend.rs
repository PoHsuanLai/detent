//! What an AI account has cost this month against the cap the person set (inferd's
//! `ai.spend.account_monthly_cents`), as the module's read-only `spend.month` and `spend.cap`
//! rows say it: whole US cents, written as digits.

/// An amount of money in whole US cents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cents(pub u64);

impl Cents {
    /// The amount a row's digits spell; none for anything else.
    pub(super) fn parse(word: &str) -> Option<Cents> {
        word.parse().ok().map(Cents)
    }

    /// "$12.40".
    pub fn words(self) -> String {
        format!("${}.{:02}", self.0 / 100, self.0 % 100)
    }
}

/// How the month stands against the cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpendBar {
    /// How much of the cap is used, in thousandths; a month over the cap reads full.
    pub permille: u16,
    /// What the bar says beside it: "$12.40 of $50.00 this month".
    pub line: SpendLine,
}

/// The words beside the bar, kept as the two amounts so a test reads them whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpendLine {
    /// Spent so far.
    pub spent: Cents,
    /// The cap.
    pub cap: Cents,
}

impl SpendLine {
    /// "$12.40 of $50.00 this month".
    pub fn words(self) -> String {
        format!("{} of {} this month", self.spent.words(), self.cap.words())
    }
}

/// The two rows an AI account may publish.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccountSpend {
    /// This month's cost.
    pub spent: Option<Cents>,
    /// The monthly cap.
    pub cap: Option<Cents>,
}

impl AccountSpend {
    /// The bar, when both rows are there and the cap is more than nothing: without a cap there is
    /// nothing to measure against.
    pub fn bar(&self) -> Option<SpendBar> {
        let (spent, cap) = (self.spent?, self.cap?);
        (cap.0 > 0).then(|| {
            let permille = (u128::from(spent.0) * 1000 / u128::from(cap.0)).min(1000);
            SpendBar {
                permille: u16::try_from(permille).unwrap_or(1000),
                line: SpendLine { spent, cap },
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountSpend, Cents};

    #[test]
    fn the_bar_is_the_share_of_the_cap_and_waits_for_both_rows() {
        let spend = |spent: Option<u64>, cap: Option<u64>| AccountSpend {
            spent: spent.map(Cents),
            cap: cap.map(Cents),
        };
        let cases = [
            (
                spend(Some(1240), Some(5000)),
                Some((248, "$12.40 of $50.00 this month")),
            ),
            (
                spend(Some(9000), Some(5000)),
                Some((1000, "$90.00 of $50.00 this month")),
            ),
            (
                spend(Some(0), Some(5000)),
                Some((0, "$0.00 of $50.00 this month")),
            ),
            (spend(Some(10), Some(0)), None),
            (spend(None, Some(5000)), None),
            (spend(Some(10), None), None),
        ];
        for (held, want) in cases {
            let got = held.bar().map(|bar| (bar.permille, bar.line.words()));
            assert_eq!(
                got,
                want.map(|(share, words)| (share, words.to_owned())),
                "{held:?}"
            );
        }
        assert_eq!(Cents::parse("1240"), Some(Cents(1240)));
        assert_eq!(Cents::parse("12.40"), None);
    }
}
