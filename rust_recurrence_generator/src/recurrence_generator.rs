use chrono::{DateTime, ParseError};

use rrule::{RRuleError, RRuleSet, Tz};

pub struct RecurrenceGenerator;

impl RecurrenceGenerator {
    // Google limits their recurrence generations to 730 instances
    // See: https://support.google.com/calendar/thread/51073472/daily-recurring-event-has-stopped-recurring
    const MAX_OCCURRENCES_COUNT: u16 = 730;

    pub(crate) fn recurrence_dates_between(
        rules: &str,
        after: &str,
        before: &str,
    ) -> Result<Vec<DateTime<Tz>>, RecurrenceGeneratorError> {
        let parsed_rule_set: Result<RRuleSet, RRuleError> = rules.parse();

        match (
            parsed_rule_set,
            RecurrenceGenerator::parse_date(after),
            RecurrenceGenerator::parse_date(before),
        ) {
            (Ok(rule_set), Ok(after), Ok(before)) => {
                let dates = rule_set
                    .after(after)
                    .before(before)
                    .all(Self::MAX_OCCURRENCES_COUNT)
                    .dates;
                return Ok(dates);
            }
            (_, _, _) => return Err(RecurrenceGeneratorError::Parsing),
        }
    }

    fn parse_date(string: &str) -> Result<DateTime<Tz>, ParseError> {
        match DateTime::parse_from_rfc3339(string) {
            Ok(datetime) => Ok(datetime.with_timezone(&Tz::UTC)),
            Err(e) => Err(e),
        }
    }
}

pub enum RecurrenceGeneratorError {
    Parsing,
}
