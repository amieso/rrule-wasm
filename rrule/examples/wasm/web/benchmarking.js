import init, { getAllRecurrencesBetween } from '../../../pkg/web/rrule.js';
import { tryParseEventRecurrenceRules, createValidDateTimeFromISO, getInstanceStartAt } from './rrule_utils.js';

function executeRRulePerformanceTest(ruleSet, after, before, limit) {
  return executeWork(() => {
    return (new rrule.rrulestr(ruleSet)).between(after, before);
  }, "rrule");
}
function executeRustRRulePerformanceTest(ruleSet, after, before, limit) {
  return executeWork(() => {
    return getAllRecurrencesBetween(ruleSet, after, before, limit);
  }, "rust-rrule");
}

const performance = window.performance;

function executeWork(work, framework, rounds = 100) {
  const measurements = [];

  for (let round = 0; round < rounds; round++) {
    const t0 = performance.now();
    work();
    const t1 = performance.now();
    measurements.push(t1 - t0);
  }

  // Calculate mean
  const mean = measurements.reduce((a, b) => a + b, 0) / measurements.length;

  // Calculate standard deviation
  const standardDeviation = Math.sqrt(
    measurements.map(x => Math.pow(x - mean, 2)).reduce((a, b) => a + b) / measurements.length
  );

  // Calculate confidence interval (95% confidence level)
  const zScore = 1.96; // Z-score for 95% confidence
  const marginOfError = zScore * (standardDeviation / Math.sqrt(measurements.length));
  const confidenceInterval = [mean - marginOfError, mean + marginOfError];

  return `Call to ${framework} took an average of ${mean.toFixed(2)} milliseconds with a 95% confidence interval of (${confidenceInterval[0].toFixed(2)}, ${confidenceInterval[1].toFixed(2)}) milliseconds.`;
}

init();

function executePerformanceTests() {
  const ruleSet = document.getElementById("ruleSet").value.replaceAll('\\n', '\n');
  const afterDateString = document.getElementById("after").value;
  const beforeDateString = document.getElementById("before").value;
  const limit = document.getElementById("limit").value;

  let after = new Date(afterDateString);
  let before = new Date(beforeDateString)

  const rustRRuleResultDiv = document.querySelector("#rustRRuleResult");
  rustRRuleResultDiv.innerHTML = "Executing ...";
  rustRRuleResultDiv.innerHTML = executeRustRRulePerformanceTest(ruleSet, after, before, limit);

  setTimeout(() => {
    const rruleResultDiv = document.querySelector("#rruleResult");
    rruleResultDiv.innerHTML = "Executing ...";
    rruleResultDiv.innerHTML = executeRRulePerformanceTest(ruleSet, after, before, limit);

    const matchErrorsDiv = document.querySelector("#matchErrors");

    // const sourceEvent = {
    //   startAt: '2023-05-31T20:00:00+05:30',
    //   startTimeZone: 'Asia/Kolkata',
    //   recurrence: [
    //     // 'DTSTART;TZID=Asia/Kolkata:20230531T200000',
    //     'EXDATE;TZID=Asia/Kolkata:20230810T200000',
    //     'RRULE:FREQ=DAILY;UNTIL=20230818T143000Z',
    //   ],
    // };
    //
    // after = new Date('2023-05-30T00:00:00Z');
    // before = new Date('2023-09-01T00:00:00Z');

    const sourceEvent = {
      startAt: '2019-08-13T15:30:00',
      startTimeZone: 'Europe/Moscow',
      recurrence: [
        // 'DTSTART;TZID=Europe/Moscow:20190813T153000',
        'RRULE:FREQ=DAILY',
      ],
    };

    after = new Date('2019-06-05T21:00:00Z');
    before = new Date('2022-06-22T20:59:59Z');

    const event = {
      recurrenceRules: sourceEvent.recurrence,
      startAt: createValidDateTimeFromISO(sourceEvent.startAt, {
        zone: sourceEvent.startTimeZone,
      }),
    }

    const rruleSet = tryParseEventRecurrenceRules(event, { useStartDate: true })

    console.log(rruleSet.toString());

    const dates1 = getAllRecurrencesBetween([
      // `DTSTART;TZID=Asia/Kolkata:20230531T200000`,
      'DTSTART;TZID=Europe/Moscow:20190813T153000',
      ...sourceEvent.recurrence].join('\n'), after, before, limit);
    const dates2 = rruleSet.between(after, before);

    console.log(dates1, dates2);

    let isFullMatch = true;

    for (let i = 0; i < dates1.length; i++) {
      let d1 = dates1.at(i);
      let d2 = dates2.at(i);

      d1 = d1 ? new Date(d1) : null;
      d2 = getInstanceStartAt(d2, event.startAt).toJSDate();

      if (d1?.getTime() !== d2?.getTime()) {
        matchErrorsDiv.innerHTML += `<li>Dates do not match at index ${i}: ${d1?.toISOString()} !== ${d2?.toISOString()}</li>`;
        isFullMatch = false;
      }
    }

    if (isFullMatch) {
      matchErrorsDiv.innerHTML = `All dates match! (${dates1.length})`;
    }
  });
}

document.addEventListener("DOMContentLoaded", () => {
  const performanceButton = document.querySelector("#performanceButton");

  performanceButton.addEventListener("click", () => {
    executePerformanceTests();
  });
});