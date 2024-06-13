const { get_all_date_recurrences_between } = require('../../../pkg/nodejs/rrule.js');

const rule_set = 'DTSTART:20120201T093000Z\nRRULE:FREQ=DAILY';
const data = get_all_date_recurrences_between(
  rule_set,
  10,
  new Date(2021, 0, 1),
  new Date(2023, 0, 1)
);

console.log(data);