const { getAllRecurrencesBetween } = require('../../../pkg/nodejs/rrule.js');

const rule_set = [
  'DTSTART:20120201T093000Z',
  'RRULE:FREQ=DAILY;UNTIL=20120201T090000Z'
].join('\n')
const data = getAllRecurrencesBetween(
  rule_set,
  new Date(2012, 0, 1).toISOString(),
  new Date(2013, 0, 1).toISOString(),
  10,
);

console.log(data);