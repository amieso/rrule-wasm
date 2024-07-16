use crate::tests::common;
use crate::{Frequency, RRule, RRuleSet, Tz, Unvalidated};
use chrono::{DateTime, Month, TimeZone};
use std::env;

#[test]
fn issue_34() {
    let dates = "DTSTART;TZID=America/New_York:19970929T090000
RRULE:FREQ=MONTHLY;BYDAY=MO,TU,WE,TH,FR;BYSETPOS=-2"
        .parse::<RRuleSet>()
        .unwrap()
        .all(7)
        .dates;
    common::check_occurrences(
        &dates,
        &[
            "1997-09-29T09:00:00-04:00",
            "1997-10-30T09:00:00-05:00",
            "1997-11-27T09:00:00-05:00",
            "1997-12-30T09:00:00-05:00",
            "1998-01-29T09:00:00-05:00",
            "1998-02-26T09:00:00-05:00",
            "1998-03-30T09:00:00-05:00",
        ],
    );
}

#[test]
fn issue_49() {
    let rrule_set = "DTSTART:20211214T091500\nEXDATE:20211228T091500,20220104T091500\nRRULE:FREQ=WEEKLY;UNTIL=20220906T091500;INTERVAL=1;BYDAY=TU;WKST=MO"
        .parse::<RRuleSet>()
        .expect("The RRule is not valid");

    let res = rrule_set.all(1).dates;
    assert!(!res.is_empty());
    let res_str = format!("{}", res[0]);
    // Check that result datetime is not in UTC
    assert!(!res_str.contains("UTC"));
}

#[test]
fn issue_61() {
    let rrule_set = "DTSTART;TZID=Europe/Berlin:18930401T010000\nRRULE:FREQ=DAILY"
        .parse::<RRuleSet>()
        .expect("The RRule is not valid");

    let res = rrule_set.all(10).dates;
    assert_eq!(res.len(), 10);
}

// Frequency should be capitalized
#[test]
fn issue_97() {
    let rrule = RRule::new(Frequency::Yearly)
        .by_month_day((24..=26).collect())
        .by_month(&[Month::December]);

    assert_eq!(
        rrule.to_string(),
        "FREQ=YEARLY;BYMONTH=12;BYMONTHDAY=24,25,26"
    );
}

#[test]
fn issue_111() {
    let rrule = "RRULE:FREQ=WEEKLY;INTERVAL=1;BYDAY=TU;WKST=SU".parse::<RRule<Unvalidated>>();

    // Convert to string...
    let rrule_str = format!("{}", rrule.unwrap());
    assert!(rrule_str.contains("WKST=SU"));
}

#[test]
fn issue_119_return_correct_number_of_instances() {
    let dates = "DTSTART;TZID=Europe/Berlin:20240530T200000\nRDATE;TZID=Europe/Berlin:20240530T200000\nRRULE:FREQ=WEEKLY;COUNT=3;INTERVAL=1;BYDAY=WE"
        .parse::<RRuleSet>()
        .unwrap()
        .all(100)
        .dates;

    common::check_occurrences(
        &dates,
        &[
            "2024-05-30T20:00:00+02:00",
            "2024-06-05T20:00:00+02:00",
            "2024-06-12T20:00:00+02:00",
            "2024-06-19T20:00:00+02:00",
        ],
    );
}

#[test]
fn issue_119_deduplicate() {
    let dates = "DTSTART;TZID=Europe/Berlin:20240530T200000\nRDATE;TZID=Europe/Berlin:20240530T200000\nRRULE:FREQ=DAILY;COUNT=3"
        .parse::<RRuleSet>()
        .unwrap()
        .all(100)
        .dates;

    common::check_occurrences(
        &dates,
        &[
            "2024-05-30T20:00:00+02:00",
            "2024-05-31T20:00:00+02:00",
            "2024-06-01T20:00:00+02:00",
        ],
    );
}

#[test]
fn issue_119_limit_correctly_when_dtstart_is_synchronized_with_rule() {
    let dates = "DTSTART;TZID=Europe/Berlin:20240530T200000\nRDATE;TZID=Europe/Berlin:20240530T200000\nRRULE:FREQ=DAILY"
        .parse::<RRuleSet>()
        .unwrap()
        .all(730)
        .dates;

    assert_eq!(dates.len(), 730);
}

#[test]
fn issue_119_limit_correctly_when_dtstart_is_not_synchronized_with_rule() {
    let dates = "DTSTART;TZID=Europe/Berlin:20240530T200000\nRDATE;TZID=Europe/Berlin:20240530T200000\nRRULE:FREQ=WEEKLY;INTERVAL=1;BYDAY=WE"
        .parse::<RRuleSet>()
        .unwrap()
        .all(730)
        .dates;

    assert_eq!(dates.len(), 730);
}

#[test]
fn issue_until_is_all_day_but_rule_is_not() {
    let dates = "DTSTART;TZID=Europe/Stockholm:20211217T120000\nRDATE;TZID=Europe/Stockholm:20211217T120000\nRRULE:FREQ=WEEKLY;WKST=SU;UNTIL=20220112;INTERVAL=2;BYDAY=FR,WE"
        .parse::<RRuleSet>()
        .unwrap()
        .after(DateTime::parse_from_rfc3339("2021-12-01T00:00:00+02:00").unwrap().with_timezone(&Tz::UTC))
        .before(DateTime::parse_from_rfc3339("2022-02-01T23:59:59+02:00").unwrap().with_timezone(&Tz::UTC))
        .all(730)
        .dates;

    assert_eq!(dates.len(), 3);

    common::check_occurrences(
        &dates,
        &[
            "2021-12-17T12:00:00+01:00",
            "2021-12-29T12:00:00+01:00",
            "2021-12-31T12:00:00+01:00",
        ],
    );
}

#[test]
fn issue_until_is_all_day_but_rule_is_not_utc_timezone() {
    let dates = "DTSTART:20211217T120000Z\nRDATE:20211217T120000Z\nRRULE:FREQ=WEEKLY;WKST=SU;UNTIL=20220112;INTERVAL=2;BYDAY=FR,WE"
        .parse::<RRuleSet>()
        .unwrap()
        .after(DateTime::parse_from_rfc3339("2021-12-01T00:00:00+02:00").unwrap().with_timezone(&Tz::UTC))
        .before(DateTime::parse_from_rfc3339("2022-02-01T23:59:59+02:00").unwrap().with_timezone(&Tz::UTC))
        .all(730)
        .dates;

    common::check_occurrences(
        &dates,
        &[
            "2021-12-17T12:00:00+00:00",
            "2021-12-29T12:00:00+00:00",
            "2021-12-31T12:00:00+00:00",
        ],
    );
}

#[test]
fn issue_until_is_all_day_but_rule_is_not_local_timezone() {
    let dates = "DTSTART:20211217T120000\nRDATE:20211217T120000\nRRULE:FREQ=WEEKLY;WKST=SU;UNTIL=20220112;INTERVAL=2;BYDAY=FR,WE"
        .parse::<RRuleSet>()
        .unwrap()
        .after(DateTime::parse_from_rfc3339("2021-12-01T00:00:00+02:00").unwrap().with_timezone(&Tz::UTC))
        .before(DateTime::parse_from_rfc3339("2022-02-01T23:59:59+02:00").unwrap().with_timezone(&Tz::UTC))
        .all(730)
        .dates;

    assert_eq!(
        dates,
        &[
            Tz::LOCAL.with_ymd_and_hms(2021, 12, 17, 12, 0, 0).unwrap(),
            Tz::LOCAL.with_ymd_and_hms(2021, 12, 29, 12, 0, 0).unwrap(),
            Tz::LOCAL.with_ymd_and_hms(2021, 12, 31, 12, 0, 0).unwrap(),
        ]
    )
}

#[test]
fn issue_local_asia_almaty_ambiguous_date_on_timezone_change() {
    with_timezone("Asia/Almaty", || {
        let dates: Vec<DateTime<Tz>> = "DTSTART:20240301\nRDATE:20240301\nRRULE:FREQ=YEARLY"
            .parse::<RRuleSet>()
            .unwrap()
            .after(
                DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00")
                    .unwrap()
                    .with_timezone(&Tz::UTC),
            )
            .before(
                DateTime::parse_from_rfc3339("2025-01-01T00:00:00+00:00")
                    .unwrap()
                    .with_timezone(&Tz::UTC),
            )
            .all(730)
            .dates;

        common::check_occurrences(&dates, &["2024-03-01T00:00:00+06:00"]);
    });
}

#[test]
fn issue_local_timezone_america_edmonton_ambiguous_date_on_dst_switch_off() {
    with_timezone("America/Edmonton", || {
        let dates: Vec<DateTime<Tz>> =
            "DTSTART:20201101T010000\nRDATE:20201101T010000\nRRULE:FREQ=YEARLY"
                .parse::<RRuleSet>()
                .unwrap()
                .after(
                    DateTime::parse_from_rfc3339("2020-01-01T00:00:00+00:00")
                        .unwrap()
                        .with_timezone(&Tz::UTC),
                )
                .before(
                    DateTime::parse_from_rfc3339("2021-01-01T00:00:00+00:00")
                        .unwrap()
                        .with_timezone(&Tz::UTC),
                )
                .all(730)
                .dates;

        common::check_occurrences(&dates, &["2020-11-01T01:00:00-06:00"]);
    });
}

#[test]
fn issue_america_edmonton_ambiguous_date_on_dst_switch_off() {
    let dates: Vec<DateTime<Tz>> =
        "DTSTART;TZID=America/Edmonton:20201101T010000\nRDATE;TZID=America/Edmonton:20201101T010000\nRRULE:FREQ=YEARLY"
            .parse::<RRuleSet>()
            .unwrap()
            .after(
                DateTime::parse_from_rfc3339("2020-01-01T00:00:00+00:00")
                    .unwrap()
                    .with_timezone(&Tz::UTC),
            )
            .before(
                DateTime::parse_from_rfc3339("2021-01-01T00:00:00+00:00")
                    .unwrap()
                    .with_timezone(&Tz::UTC),
            )
            .all(730)
            .dates;

    common::check_occurrences(&dates, &["2020-11-01T01:00:00-06:00"]);
}

#[test]
fn by_month_day_in_weekly_freq() {
    let rrule_set = "DTSTART:20230101T000000Z\nRRULE:FREQ=WEEKLY;BYMONTHDAY=25;BYWEEKDAY=SU"
        .parse::<RRuleSet>()
        .unwrap();

    let rrule: &RRule = &rrule_set.get_rrule()[0];

    assert_eq!(rrule.by_month_day.is_empty(), true);

    let dates = rrule_set.all(5).dates;

    common::check_occurrences(&dates, &[
        "2023-01-01T00:00:00+00:00",
        "2023-01-08T00:00:00+00:00",
        "2023-01-15T00:00:00+00:00",
        "2023-01-22T00:00:00+00:00",
        "2023-01-29T00:00:00+00:00",
    ]);
}

#[test]
fn by_year_day_in_monthly_freq() {
    let rrule_set = "DTSTART:20230101T000000Z\nRRULE:FREQ=MONTHLY;BYYEARDAY=25;BYMONTHDAY=25"
        .parse::<RRuleSet>()
        .unwrap();

    let rrule: &RRule = &rrule_set.get_rrule()[0];

    assert_eq!(rrule.by_year_day.is_empty(), true);

    let dates = rrule_set.all(5).dates;

    common::check_occurrences(&dates, &[
        "2023-01-25T00:00:00+00:00",
        "2023-02-25T00:00:00+00:00",
        "2023-03-25T00:00:00+00:00",
        "2023-04-25T00:00:00+00:00",
        "2023-05-25T00:00:00+00:00",
    ]);
}

#[test]
fn by_year_day_in_weekly_freq() {
    let rrule_set = "DTSTART:20230101T000000Z\nRRULE:FREQ=WEEKLY;BYYEARDAY=25;BYWEEKDAY=SU"
        .parse::<RRuleSet>()
        .unwrap();

    let rrule: &RRule = &rrule_set.get_rrule()[0];

    assert_eq!(rrule.by_year_day.is_empty(), true);

    let dates = rrule_set.all(5).dates;

    common::check_occurrences(&dates, &[
        "2023-01-01T00:00:00+00:00",
        "2023-01-08T00:00:00+00:00",
        "2023-01-15T00:00:00+00:00",
        "2023-01-22T00:00:00+00:00",
        "2023-01-29T00:00:00+00:00",
    ]);
}

#[test]
fn by_year_day_in_daily_freq() {
    let rrule_set = "DTSTART:20230101T000000Z\nRRULE:FREQ=DAILY;BYYEARDAY=25"
        .parse::<RRuleSet>()
        .unwrap();

    let rrule: &RRule = &rrule_set.get_rrule()[0];

    assert_eq!(rrule.by_year_day.is_empty(), true);

    let dates = rrule_set.all(5).dates;

    common::check_occurrences(&dates, &[
        "2023-01-01T00:00:00+00:00",
        "2023-01-02T00:00:00+00:00",
        "2023-01-03T00:00:00+00:00",
        "2023-01-04T00:00:00+00:00",
        "2023-01-05T00:00:00+00:00",
    ]);
}

#[test]
fn duplicate_property_for_until_should_use_latest_one() {
    let rrule_set = "DTSTART:20230101T000000Z\nRRULE:UNTIL=20280108T145959Z;FREQ=WEEKLY;BYDAY=SU;UNTIL=20230114T145959Z"
        .parse::<RRuleSet>()
        .unwrap();

    let rrule = &rrule_set.get_rrule()[0];

    assert_eq!(rrule.until, Some(common::ymd_hms(2023, 01, 14, 14, 59, 59)));

    let dates = rrule_set.all(1000).dates;

    common::check_occurrences(&dates, &[
        "2023-01-01T00:00:00+00:00",
        "2023-01-08T00:00:00+00:00"
    ]);
}

#[test]
fn issue_ignore_3rd_party_params() {
    let dates = "DTSTART;TZID=Europe/Berlin:20201101T010000\nRRULE;X-BUSYMAC-REGENERATE=TRASH:FREQ=MONTHLY"
        .parse::<RRuleSet>()
        .unwrap()
        .all(1)
        .dates;

    common::check_occurrences(&dates, &["2020-11-01T01:00:00+01:00"]);
}

fn with_timezone<F: FnOnce()>(tz: &str, test: F) {
    // Save the current timezone to restore it later
    let original_tz = env::var("TZ").ok();

    // Set the new timezone
    env::set_var("TZ", tz);
    // Force chrono to update its internal timezone cache
    chrono::Local::now();

    // Run the test
    test();

    // Restore the original timezone
    if let Some(tz) = original_tz {
        env::set_var("TZ", tz);
    } else {
        env::remove_var("TZ");
    }
    // Force chrono to update its internal timezone cache again
    chrono::Local::now();
}
