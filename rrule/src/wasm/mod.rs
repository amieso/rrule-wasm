use wasm_bindgen::prelude::*;
use chrono::{DateTime};
use crate::{RRuleSet, RRuleError};
use crate::{core::Tz};

const MAX_OCCURRENCES_COUNT: u16 = 730;
const MAX_RESULT_LIMIT: u16 = 1000;

/// When the `console_error_panic_hook` feature is enabled, we can call the
/// `set_panic_hook` function at least once during initialization, and then
/// we will get better error messages if our code ever panics.
///
/// For more details see
/// https://github.com/rustwasm/console_error_panic_hook#readme
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Get all recurrences of the rrule
#[wasm_bindgen(js_name = getAllRecurrencesBetween)]
pub fn get_all_recurrences_between(rules: &str, after: &str, before: &str, count: Option<u32>) -> Result<Vec<JsValue>, JsError> {
    set_panic_hook();

    let after = parse_date(after);
    let before = parse_date(before);

    match (parser_rrule_set(rules), after, before) {
        (Ok(rrule_set), Ok(after), Ok(before)) => {
            let mut cloned_rrules = rrule_set.get_rrule().clone();
            let max_count: u32 = MAX_OCCURRENCES_COUNT.into();

            cloned_rrules.iter_mut().for_each(|rrule| {
                // Handle the limit of the number of recurrences
                if let Some(count) = count {
                    // If count is provided, use it.
                    rrule.count = Some(count);
                } else if rrule.count.is_none() || rrule.count.unwrap() > max_count {
                    // Otherwise, use the max count.
                    rrule.count = Some(max_count);
                }
            });

            let final_rrule_set = rrule_set.set_rrules(cloned_rrules).after(after).before(before);

            Ok(get_all_recurrences_for(final_rrule_set))
        }
        (Err(e), _, _) => Err(e),
        (_, Err(e), _) => Err(e),
        (_, _, Err(e)) => Err(e)
    }
}

fn parse_date(date: &str) -> Result<DateTime<Tz>, JsError> {
    match DateTime::parse_from_rfc3339(date) {
        Ok(datetime) => Ok(datetime.with_timezone(&Tz::UTC)),
        Err(e) => Err(JsError::from(e)),
    }
}

fn parser_rrule_set(rules: &str) -> Result<RRuleSet, JsError> {
    let rrule_set_result: Result<RRuleSet, RRuleError> = rules.parse();

    match rrule_set_result {
        Ok(rrule_set) => Ok(rrule_set),
        Err(e) => Err(JsError::from(e))
    }
}

fn get_all_recurrences_for(rrule_set: RRuleSet) -> Vec<JsValue> {
    let rrule_set_collection = rrule_set.all(MAX_RESULT_LIMIT);
    let mut result = Vec::new();

    for dt in rrule_set_collection.dates {
        result.push(JsValue::from_str(&dt.to_rfc3339()));
    }

    result
}
