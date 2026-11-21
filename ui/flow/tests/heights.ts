import { expect, test } from "vitest";
import { RowHeights } from "../heights";

const ESTIMATE = 46;
const MEASURED_TALLER = 100;
const FIRST = 0;
const SECOND = 1;
const THIRD = 2;
const FOURTH = 3;
const FIFTH = 4;
const FIVE_ROWS = 5;

test("rows start at the estimated height", () => {
  const heights = new RowHeights(FIVE_ROWS, ESTIMATE);

  expect(heights.total()).toBe(FIVE_ROWS * ESTIMATE);
  expect(heights.offsetOf(FOURTH)).toBe(FOURTH * ESTIMATE);
});

test("the row at an offset is the one that offset falls in", () => {
  const heights = new RowHeights(FIVE_ROWS, ESTIMATE);

  expect(heights.rowAt(0)).toBe(FIRST);
  expect(heights.rowAt(FOURTH * ESTIMATE - 1)).toBe(THIRD);
  expect(heights.rowAt(FOURTH * ESTIMATE)).toBe(FOURTH);
  expect(heights.rowAt(FIVE_ROWS * ESTIMATE * FIVE_ROWS)).toBe(FIFTH);
});

test("a measured row moves every row after it", () => {
  const heights = new RowHeights(FIVE_ROWS, ESTIMATE);

  const change = heights.set(THIRD, MEASURED_TALLER);

  const aboveTheFourth = THIRD * ESTIMATE + MEASURED_TALLER;
  expect(change).toBe(MEASURED_TALLER - ESTIMATE);
  expect(heights.total()).toBe(FIFTH * ESTIMATE + MEASURED_TALLER);
  expect(heights.offsetOf(FOURTH)).toBe(aboveTheFourth);
  expect(heights.rowAt(aboveTheFourth - 1)).toBe(THIRD);
  expect(heights.heightOf(THIRD)).toBe(MEASURED_TALLER);
});

test("measuring a row at the height it has changes nothing", () => {
  const heights = new RowHeights(FIVE_ROWS, ESTIMATE);

  expect(heights.set(SECOND, ESTIMATE)).toBe(0);
  expect(heights.total()).toBe(FIVE_ROWS * ESTIMATE);
});

test("more rows keep the heights already measured", () => {
  const heights = new RowHeights(THIRD, ESTIMATE);
  heights.set(FIRST, MEASURED_TALLER);

  const grown = heights.resized(FIVE_ROWS);

  expect(grown.total()).toBe(MEASURED_TALLER + FIFTH * ESTIMATE);
  expect(grown.heightOf(FIRST)).toBe(MEASURED_TALLER);
});

test("an empty list is no height at all", () => {
  const heights = new RowHeights(0, ESTIMATE);

  expect(heights.total()).toBe(0);
  expect(heights.rowAt(MEASURED_TALLER)).toBe(FIRST);
});

test("a row is measured once, until the measurements are forgotten", () => {
  const heights = new RowHeights(FIVE_ROWS, ESTIMATE);
  expect(heights.isMeasured(THIRD)).toBe(false);

  heights.set(THIRD, MEASURED_TALLER);
  const grown = heights.resized(FIVE_ROWS + 1);
  expect(heights.isMeasured(THIRD)).toBe(true);
  expect(grown.isMeasured(THIRD)).toBe(true);

  heights.forgetMeasurements();
  expect(heights.isMeasured(THIRD)).toBe(false);
  expect(heights.heightOf(THIRD)).toBe(MEASURED_TALLER);
});

test("one row's measurement can be forgotten, so it is measured again", () => {
  const heights = new RowHeights(FIVE_ROWS, ESTIMATE);
  heights.set(SECOND, MEASURED_TALLER);
  heights.set(THIRD, MEASURED_TALLER);

  heights.forgetMeasurementOf(THIRD);

  expect(heights.isMeasured(SECOND)).toBe(true);
  expect(heights.isMeasured(THIRD)).toBe(false);
});
