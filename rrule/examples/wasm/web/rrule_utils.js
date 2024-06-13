import { DateTime } from './node_modules/luxon/build/es6/luxon.js';

const rrulestr = rrule.rrulestr;

/** The max number of recurring events (2 years) */
export const MAX_OCCURRENCES_COUNT = 730;
const DEFAULT_TZID = 'utc';
const TZID_REGEX = /;TZID=([^;:]+)/;
const DATE_FORMAT = `yyyyMMdd'T'HHmmss`;
const DATE_FORMAT_UTC = `yyyyMMdd'T'HHmmss'Z'`;
export function parseRecurrenceRules(rules, options) {
  var _a, _b, _c, _d;
  const dtstart = options === null || options === void 0 ? void 0 : options.dtstart;
  const tzid = options === null || options === void 0 ? void 0 : options.tzid;
  const count = (_a = options === null || options === void 0 ? void 0 : options.count) !== null && _a !== void 0 ? _a : 0;
  const timeZone = (_b = tzid === null || tzid === void 0 ? void 0 : tzid.toLowerCase()) !== null && _b !== void 0 ? _b : DEFAULT_TZID;
  if (dtstart) {
    rules = [
      `DTSTART${formatRuleTzid(tzid)}:${formatDateInZone(dtstart, timeZone)}`,
      ...rules,
    ];
  }
  for (let i = 0; i < rules.length; i++) {
    const rule = rules[i];
    if (rule.startsWith('RDATE') || rule.startsWith('EXDATE')) {
      const [key, value] = rule.split(':');
      const ruleZone = (_d = (_c = key.match(TZID_REGEX)) === null || _c === void 0 ? void 0 : _c[1]) !== null && _d !== void 0 ? _d : 'utc';
      if (ruleZone.toLowerCase() !== timeZone) {
        const filteredKey = key.replace(TZID_REGEX, '');
        const adjustedValues = value
          .split(',')
          .map((date) => reformatDateInZone(date, timeZone, ruleZone))
          .join(',');
        rules[i] = `${filteredKey}:${adjustedValues}`;
      }
    }
  }
  const rruleSet = rrulestr(sanitizeRecurrenceRules(rules), {
    compatible: true,
    cache: false,
    dtstart,
    tzid,
  });
  for (const rrule of rruleSet._rrule) {
    const options = rrule.options;
    if (count > 0) {
      // Hard limit the number of generated instances to the provided count.
      options.count = count;
    }
    else {
      // Limit the number of generated instances to 2 years for daily events.
      // See: https://support.google.com/calendar/thread/51073472/daily-recurring-event-has-stopped-recurring
      options.count = options.count
        ? Math.min(options.count, MAX_OCCURRENCES_COUNT)
        : MAX_OCCURRENCES_COUNT;
    }
    if (options.until && tzid) {
      // TODO(@vk): Should we do the same for all day events?
      // UNTIL date usually represented in UTC timezone. so we need to convert it to the specified timezone (tzid).
      options.until = transformDateToZone(options.until, 'utc', tzid);
    }
  }
  return rruleSet;
}
export function tryParseRecurrenceRules(rules, options) {
  try {
    return parseRecurrenceRules(rules, options);
  }
  catch (error) {
    console.error('Failed to parse recurrence rules', rules, error);
  }
}
/**
 * Normalizes recurrence rules by filtering out unsupported rules.
 */
export function sanitizeRecurrenceRules(rules) {
  return rules.join('\n').replaceAll(/;X-EVOLUTION-ENDDATE=\d{8}T\d{6}Z/gm, '');
}
export function tryParseEventRecurrenceRules(event, options) {
  if (!(event === null || event === void 0 ? void 0 : event.recurrenceRules)) {
    return;
  }
  const parseOptions = {};
  if (options === null || options === void 0 ? void 0 : options.useStartDate) {
    if (event.startDate /*&& event.endDate*/) {
      parseOptions.dtstart = new Date(event.startDate);
    }
    else if (event.startAt /*&& event.endAt*/) {
      parseOptions.dtstart = event.startAt.setZone('system').toJSDate();
      parseOptions.tzid = event.startAt.zoneName;
    }
  }
  if (options === null || options === void 0 ? void 0 : options.count) {
    parseOptions.count = options.count;
  }
  return tryParseRecurrenceRules(event.recurrenceRules, parseOptions);
}

export function normalizeRecurringRules(rruleSet, isAllDay = false) {
  const rules = rruleSet.valueOf();
  if (!isAllDay) {
    return rules;
  }
  for (let i = 0; i < rules.length; i++) {
    let rule = rules[i];
    if (rule.startsWith('EXDATE')) {
      rule = rule
        .replace('EXDATE', 'EXDATE;VALUE=DATE')
        .replaceAll('T000000Z', '');
      rules[i] = rule;
    }
    else if (rule.startsWith('RDATE')) {
      rule = rule
        .replace('RDATE', 'RDATE;VALUE=DATE')
        .replaceAll('T000000Z', '');
      rules[i] = rule;
    }
    else if (rule.startsWith('RRULE')) {
      rule = rule.replaceAll('T000000Z', '');
      rules[i] = rule;
    }
  }
  return rules;
}
export function getStartUTCDate(date) {
  return date.setZone('utc').toJSDate();
}
export function getInstanceStartAt(instanceDate, parentStartAt) {
  if (parentStartAt.zone.isUniversal) {
    return DateTime.fromJSDate(instanceDate);
  }
  return DateTime.fromJSDate(instanceDate)
    .toUTC()
    .setZone('system', { keepLocalTime: true })
    .setZone(parentStartAt.zone);
}
export function getInstanceStartDate(instanceDate) {
  return DateTime.fromJSDate(instanceDate)
    .toUTC()
    .setZone('system', { keepLocalTime: true });
}
export function buildCutoffUntilUTCDate(date) {
  return date
    .setZone('utc', { keepLocalTime: true })
    .minus({ days: 1 })
    .endOf('day')
    .toJSDate();
}
function reformatDateInZone(dateString, targetZone, sourceZone) {
  for (const format of [DATE_FORMAT, DATE_FORMAT_UTC]) {
    const dt = DateTime.fromFormat(dateString, format, {
      zone: sourceZone,
    });
    if (dt.isValid) {
      return dt.setZone(targetZone).toFormat(DATE_FORMAT);
    }
  }
  throw new Error(`💥 [reformatDateInZone] Invalid date string: ${dateString}`);
}
function formatRuleTzid(tzid) {
  return tzid ? `;TZID=${tzid}` : '';
}
function formatDateInZone(date, targetZone, sourceZone) {
  return DateTime.fromJSDate(date, { zone: sourceZone })
    .setZone(targetZone)
    .toFormat(DATE_FORMAT);
}
function transformDateToZone(date, targetZone, sourceZone) {
  return DateTime.fromJSDate(date, { zone: sourceZone })
    .setZone(targetZone, { keepLocalTime: true })
    .toJSDate();
}
export function createValidDateTimeFromISO(text, opts) {
  const date = DateTime.fromISO(text, opts);
  assertDateTimeValid(date);
  return date;
}
export function createValidDateTimeFromObject(obj, opts) {
  const date = DateTime.fromObject(obj, opts);
  assertDateTimeValid(date);
  return date;
}
export function createValidDateTimeFromJSDate(obj, opts) {
  const date = DateTime.fromJSDate(obj, opts);
  assertDateTimeValid(date);
  return date;
}
export function createValidDateTimeFromFormat(text, fmt, opts) {
  const date = DateTime.fromFormat(text, fmt, opts);
  assertDateTimeValid(date);
  return date;
}
export function createValidDateTimeFromMillis(millis, opts) {
  const date = DateTime.fromMillis(millis, opts);
  assertDateTimeValid(date);
  return date;
}
export function setValidDateTimeZone(date, zone, opts) {
  const dateInZone = date.setZone(zone, opts);
  assertDateTimeValid(dateInZone);
  return dateInZone;
}
export function assertDateTimeValid(date) {
  if (!date.isValid) {
    const errorMessage = date.invalidExplanation
      ? `${date.invalidReason}: ${date.invalidExplanation}`
      : date.invalidReason;
    throw new Error(`Invalid DateTime: ${errorMessage !== null && errorMessage !== void 0 ? errorMessage : 'unknown'}`);
  }
}
