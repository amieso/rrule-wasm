use super::{rrule_iter::RRuleIter, MAX_ITER_LOOP};
use crate::{core::DateTime, RRule, RRuleError, RRuleSet, WithError};
use chrono::TimeZone;
use std::collections::BTreeSet;
use std::{collections::HashMap, iter::Iterator};

#[derive(Debug, Clone)]
/// Iterator over all the dates in an [`RRuleSet`].
pub struct RRuleSetIter<'a> {
    queue: HashMap<usize, DateTime>,
    rrule_iters: Vec<RRuleIter<'a>>,
    exrules: &'a Vec<RRule>,
    exdates: BTreeSet<i64>,
    /// Sorted additional dates in descending order
    rdates: Vec<DateTime>,
    /// Store the last error, so it can be handled by the user.
    error: Option<RRuleError>,
    dt_start: DateTime,
}

impl<'a> RRuleSetIter<'a> {
    fn generate_date(
        dates: &mut Vec<DateTime>,
        exrules: &[RRule],
        exdates: &mut BTreeSet<i64>,
        dt_start: &DateTime,
    ) -> Result<Option<DateTime>, RRuleError> {
        if dates.is_empty() {
            return Ok(None);
        }

        let mut date = dates.remove(dates.len() - 1);
        let mut loop_counter: u32 = 0;
        while Self::is_date_excluded(&Some(date), exrules, exdates, dt_start) {
            if dates.is_empty() {
                return Ok(None);
            }
            // Prevent infinite loops
            loop_counter += 1;
            if loop_counter >= MAX_ITER_LOOP {
                return Err(RRuleError::new_iter_err(format!(
                    "Reached max loop counter (`{}`). \
                See 'validator limits' in docs for more info.",
                    MAX_ITER_LOOP
                )));
            }
            date = dates.remove(dates.len() - 1);
        }

        Ok(Some(date))
    }

    fn generate(
        rrule_iter: &mut RRuleIter,
        exrules: &[RRule],
        exdates: &mut BTreeSet<i64>,
        dt_start: &DateTime,
    ) -> Result<Option<DateTime>, RRuleError> {
        let mut date = rrule_iter.next();
        let mut loop_counter: u32 = 0;
        while Self::is_date_excluded(&date, exrules, exdates, dt_start) {
            // Prevent infinite loops
            loop_counter += 1;
            if loop_counter >= MAX_ITER_LOOP {
                return Err(RRuleError::new_iter_err(format!(
                    "Reached max loop counter (`{}`). \
                    See 'validator limits' in docs for more info.",
                    MAX_ITER_LOOP
                )));
            }

            date = rrule_iter.next();
        }

        Ok(date)
    }

    fn is_date_excluded(
        date: &Option<DateTime>,
        exrules: &[RRule],
        exdates: &mut BTreeSet<i64>,
        dt_start: &DateTime,
    ) -> bool {
        match date {
            None => false,
            Some(date) => {
                let ts = date.timestamp();

                if !exrules.is_empty() {
                    let start = date.timezone().timestamp(ts - 1, 0);
                    let end = date.timezone().timestamp(ts + 1, 0);
                    for exrule in exrules {
                        let ex = exrule.iter_with_ctx(*dt_start);
                        for date in ex.all_between(start, end, true).unwrap() {
                            exdates.insert(date.timestamp());
                        }
                    }
                }

                exdates.contains(&ts)
            }
        }
    }
}

impl<'a> WithError for RRuleSetIter<'a> {
    fn has_err(&self) -> bool {
        self.error.is_some()
    }

    fn get_err(&self) -> Option<&RRuleError> {
        self.error.as_ref()
    }
}

impl<'a> Iterator for RRuleSetIter<'a> {
    type Item = DateTime;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_date: Option<(usize, DateTime)> = None;

        // If there already was an error, return the error again.
        if let Some(err) = self.get_err() {
            log::error!("{}", err);
            return None;
        }

        for (i, rrule_iter) in self.rrule_iters.iter_mut().enumerate() {
            let rrule_queue = self.queue.remove(&i);
            let next_rrule_date = match rrule_queue {
                Some(d) => Some(d),
                None => {
                    // should be method on self
                    match Self::generate(
                        rrule_iter,
                        self.exrules,
                        &mut self.exdates,
                        &self.dt_start,
                    ) {
                        Ok(next_date) => next_date,
                        Err(err) => {
                            log::error!("{}", err);
                            self.error = Some(err);
                            return None;
                        }
                    }
                }
            };

            if let Some(next_rrule_date) = next_rrule_date {
                match next_date {
                    None => next_date = Some((i, next_rrule_date)),
                    Some(date) => {
                        if date.1 >= next_rrule_date {
                            // Add previous date to its rrule queue
                            self.queue.insert(date.0, date.1);

                            // Update next_date
                            next_date = Some((i, next_rrule_date));
                        } else {
                            // Store for next iterations
                            self.queue.insert(i, next_rrule_date);
                        }
                    }
                }
            }
        }

        // TODO: RDates should be prefiltered before starting iteration
        let generated_date = match Self::generate_date(
            &mut self.rdates,
            self.exrules,
            &mut self.exdates,
            &self.dt_start,
        ) {
            Ok(next_date) => next_date,
            Err(err) => {
                log::error!("{}", err);
                self.error = Some(err);
                return None;
            }
        };

        match generated_date {
            Some(first_rdate) => {
                let next_date = match next_date {
                    Some(next_date) => {
                        if next_date.1 >= first_rdate {
                            // Add previous date to its rrule queue
                            self.queue.insert(next_date.0, next_date.1);

                            first_rdate
                        } else {
                            // add rdate back
                            self.rdates.push(first_rdate);

                            next_date.1
                        }
                    }
                    None => first_rdate,
                };
                Some(next_date)
            }
            None => next_date.map(|d| d.1),
        }
    }
}

impl<'a> IntoIterator for &'a RRuleSet {
    type Item = DateTime;

    type IntoIter = RRuleSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        // Sort in decreasing order
        let mut rdates_sorted = self.rdate.clone();
        rdates_sorted
            .sort_by(|d1, d2| d2.partial_cmp(d1).expect("Could not order dates correctly"));

        RRuleSetIter {
            queue: HashMap::new(),
            rrule_iters: self
                .rrule
                .iter()
                .map(|rrule| rrule.iter_with_ctx(self.dt_start))
                .collect(),
            rdates: rdates_sorted,
            exrules: &self.exrule,
            exdates: self.exdate.iter().map(DateTime::timestamp).collect(),
            error: None,
            dt_start: self.dt_start,
        }
    }
}
