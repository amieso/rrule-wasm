mod datetime_utils;

use wasm_bindgen::prelude::*;
use crate::{RRuleSet, RRuleError};

const MAX_OCCURRENCES_COUNT: u16  = 730;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Get all recurrences of the rrule
#[wasm_bindgen(js_name = getAllRecurrencesBetween)]
pub fn get_all_recurrences_between(rrule_set_str: &str, after: js_sys::Date, before: js_sys::Date, count: Option<u32>) -> Result<Vec<JsValue>, JsError> {
    set_panic_hook();

    let after = datetime_utils::convert_js_date_to_datetime(&after).map_err(JsError::from);
    let before = datetime_utils::convert_js_date_to_datetime(&before).map_err(JsError::from);

    match (parser_rule_set(rrule_set_str), after, before) {
        (Ok(rrule_set), Ok(after), Ok(before)) => {
            let mut cloned_rrules = rrule_set.get_rrule().clone();
            let max_count: u32 = MAX_OCCURRENCES_COUNT.into();

            cloned_rrules.iter_mut().for_each(|rrule| {
                if rrule.count.is_none() || rrule.count.unwrap() > max_count {
                    rrule.count = Some(max_count);
                }
            });

            let final_rrule_set = rrule_set.set_rrules(cloned_rrules).after(after).before(before);

            Ok(get_all_recurrences_for(final_rrule_set))
        },
        (Err(e), _, _) => Err(e),
        (_, Err(e), _) => Err(e),
        (_, _, Err(e)) => Err(e),
    }
}

fn parser_rule_set(rrule_set_str: &str) -> Result<RRuleSet, JsError> {
    let rrule_set_result: Result<RRuleSet, RRuleError> = rrule_set_str.parse();

    match rrule_set_result {
        Ok(rrule_set) => Ok(rrule_set),
        Err(e) => Err(JsError::from(e))
    }
}

fn get_all_recurrences_for(rrule_set: RRuleSet) -> Vec<JsValue> {
    let rrule_set_collection = rrule_set.all(MAX_OCCURRENCES_COUNT);
    let result: Vec<JsValue> = rrule_set_collection.dates
        .into_iter()
        .map(|dt| {
            JsValue::from_str(&dt.to_rfc3339())
        })
        .collect();

    result
}