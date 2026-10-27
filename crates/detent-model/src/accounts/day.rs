//! A calendar day as the account modules write it, `YYYY-MM-DD`: the month and the day of the
//! month, which is all a person is told ("8 Oct").

/// A day of the year, by its month and date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDay {
    /// The month, 1 to 12.
    pub month: u8,
    /// The day of the month, 1 to 31.
    pub date: u8,
}

impl CalendarDay {
    /// The day `YYYY-MM-DD` names; none for text of another shape.
    pub fn parse(day: &str) -> Option<CalendarDay> {
        let mut parts = day.split('-');
        let (_year, month, date) = (parts.next()?, parts.next()?, parts.next()?);
        let month: u8 = month.parse().ok()?;
        let date: u8 = date.parse().ok()?;
        (parts.next().is_none() && (1..=12).contains(&month) && (1..=31).contains(&date))
            .then_some(CalendarDay { month, date })
    }
}

#[cfg(test)]
mod tests {
    use super::CalendarDay;

    #[test]
    fn a_day_is_read_from_year_month_date_and_nothing_else() {
        let cases = [
            ("2026-10-08", Some(CalendarDay { month: 10, date: 8 })),
            (
                "2026-12-31",
                Some(CalendarDay {
                    month: 12,
                    date: 31,
                }),
            ),
            ("2026-13-01", None),
            ("2026-00-01", None),
            ("2026-10-32", None),
            ("2026-10", None),
            ("2026-10-08-01", None),
            ("not a day", None),
        ];
        for (text, want) in cases {
            assert_eq!(CalendarDay::parse(text), want, "{text}");
        }
    }
}
