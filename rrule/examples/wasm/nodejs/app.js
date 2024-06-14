const { getAllRecurrencesBetween } = require('../../../pkg/nodejs/rrule.js');

const rule_set = 'DTSTART:20120201T093000Z\nRRULE:FREQ=DAILY';
const data = getAllRecurrencesBetween(
  rule_set,
  new Date(2021, 0, 1).toISOString(),
  new Date(2023, 0, 1).toISOString(),
  10,
);

console.log(data);